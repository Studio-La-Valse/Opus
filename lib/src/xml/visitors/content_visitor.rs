use crate::color::Color;
use crate::core::xy::XY;
use crate::duration::BaseDuration;
use crate::score::core::pitch::Pitch;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::core::step::Step;
use crate::score::visual::note::Note;
use crate::score::visual::page::Page;
use crate::smufl::glyphs::clef::Clef;
use crate::visitor::Visitor;
use crate::visual::brace::Brace;
use crate::visual::bracket::Bracket;
use crate::visual::chord::Chord;
use crate::visual::part::Part;
use crate::visual::part_group::PartGroup;
use crate::visual::part_measure::PartMeasure;
use crate::visual::rest::Rest;
use crate::visual::section::Section;
use crate::visual::stem::{BeamType, Stem, UpDown};
use crate::xml::utils::{NodeUtils, ToNumber};
use crate::xml::walker_ctx::WalkerCtx;
use roxmltree::Node;
use std::collections::{BTreeMap, HashSet};

pub struct ContentVisitor {
    pub part_measure: Option<PartMeasure>,
}

impl ContentVisitor {}

impl Visitor for ContentVisitor {
    fn enter(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_work(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_defaults(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn exit_defaults(&mut self, _ctx: &mut WalkerCtx) {}

    fn enter_part_list(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_part(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_measure(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {
        self.part_measure = Some(PartMeasure::new(_ctx.layout_ctx.measure.number));
    }

    fn enter_print(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_attributes(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_clef(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_staff_details(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_backup(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_forward(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_note(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        let staff: StaffIdx = ctx.layout_ctx.staff.number.into();
        let scale = ctx
            .layout_ctx
            .staff
            .staff_scaling
            .get(&staff)
            .unwrap_or(&1.);

        let voice = ctx.layout_ctx.voice;
        let part_measure: &mut PartMeasure = self.part_measure.as_mut().unwrap();

        let is_rest = node.children().find(|n| n.tag_name().name() == "rest");

        if let Some(rest) = is_rest {
            let is_measure = rest
                .attribute("measure")
                .map(|attr| attr == "yes")
                .unwrap_or(false);

            let rest = if is_measure {
                let notehead = duration_to_rest(&BaseDuration::Whole);
                let glyph = ctx.font.rest(notehead);
                Rest::new(glyph, is_measure, None, staff, 4, *scale)
            } else {
                let type_str = node.req_child("type");
                let dur = type_to_duration(type_str.req_text());
                let notehead = duration_to_rest(&dur);
                let glyph = ctx.font.rest(notehead);

                let default_x: f32 = node.req_attribute("default-x").req_f32();
                Rest::new(glyph, is_measure, Some(default_x), staff, 4, *scale)
            };

            part_measure.rests.push(rest);
            return;
        }

        // Parse default-x
        let default_x: f32 = match node.get_attribute("default-x") {
            Some(s) => s.parse().unwrap(),
            None => return, // no position → ignore note
        };

        // Extract <pitch>
        let pitch_node = match node.get_child("pitch") {
            Some(n) => n,
            None => return,
        };

        // Extract <step>
        let step = {
            let step_node = pitch_node.req_child("step");
            let step_str = step_node.req_text();

            let alter = pitch_node
                .get_child("alter")
                .map(|n| n.req_i32())
                .unwrap_or(0);

            Step::parse(step_str, alter)
        };

        // Extract <octave>
        let octave = pitch_node.req_child("octave").req_i32();

        // Build note
        let pitch = Pitch { step, octave };

        let clef = ctx.layout_ctx.staff.clef.get(&staff).unwrap();
        let staff_line = clef.line_index_at_pitch(&pitch);
        let type_str = node.req_child("type");

        let dur = type_to_duration(type_str.req_text());
        let notehead = duration_to_notehead(&dur);

        let glyph = ctx.font.notehead(notehead);

        let note = Note::new(glyph, default_x, staff, staff_line, *scale);
        let is_chord_node = node.children().any(|n| n.tag_name().name() == "chord");

        let chords = part_measure.chords.entry(voice).or_default();
        if !is_chord_node {
            chords.push(Chord::default())
        }

        let chord = chords.last_mut().unwrap();

        if let Some(stem) = node.children().find(|n| n.tag_name().name() == "stem") {
            let default_y = stem.attribute("default-y").map(|a| a.req_f32());

            let text = stem.req_text();
            let dir = match text {
                "up" => UpDown::Up,
                "down" => UpDown::Down,
                _ => panic!("Unknown direction {}", text),
            };
            let stem = chord
                .stem
                .get_or_insert_with(|| Stem::new(dir, dur, staff, *scale, default_y));

            let beams: Vec<Node> = node
                .children()
                .filter(|n| n.tag_name().name() == "beam")
                .collect();
            if beams.is_empty() && stem.beams.is_empty() {
                if dur.beam_count() > 0 {
                    let flag = duration_to_flag(&dur, &dir).unwrap();
                    let glyph = ctx.font.flag(flag, &dir);
                    stem.flag = Some(glyph);
                }
            } else {
                for beam in beams {
                    let number = beam.req_attribute("number").req_u32();
                    let beam_type: BeamType = beam.req_text().into();
                    stem.beams.insert(number, beam_type);
                }
            }
        }

        chord.notes.push(note)
    }

    fn exit_note(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit_measure(&mut self, ctx: &mut WalkerCtx) {
        let page_number = ctx.layout_ctx.page.page_number;
        let system_index = ctx.layout_ctx.system.index;
        let measure_number = ctx.layout_ctx.measure.number;

        let part_id = ctx.layout_ctx.part_id.clone();
        let part = ctx.layout.parts.get(&part_id).unwrap();
        let section_number = part.section;
        let part_group_number = part.part_group;
        let section = ctx.layout.sections.entry(section_number).or_default();
        let _part_group = section.groups.entry(part_group_number).or_default();

        // take the part measure from the option
        let part_measure = self.part_measure.take().unwrap();

        // get or create the page
        let page = ctx
            .visual_score
            .pages
            .entry(page_number)
            .or_insert_with(|| Page {
                number: page_number,
                xy: XY::default(),
                width: ctx.layout.defaults.page_width,
                height: ctx.layout.defaults.page_height,
                color: Color::WHITE,
                foreground: Color::BLACK,
                margins: ctx.layout.get_margins(page_number),
                systems: BTreeMap::new(),
            });

        // get or create the system on the page
        let system = page.systems.entry(system_index).or_default();
        system.m_left = ctx.layout_ctx.system.margin_left.unwrap_or(system.m_left);
        system.m_right = ctx.layout_ctx.system.margin_right.unwrap_or(system.m_right);
        system.distance = ctx.layout_ctx.system.distance.unwrap_or(system.distance);
        system.top = ctx.layout_ctx.system.distance_top.unwrap_or(system.top);

        let system_measure = system.measures.entry(measure_number).or_default();
        system_measure.init_width(ctx.layout_ctx.measure.width);

        // get or create the section in this system.
        let section = system.sections.entry(section_number).or_insert_with(|| {
            let bracket_top = ctx.font.bracket_top();
            let bracket_bottom = ctx.font.bracket_bottom();

            let bracket = Bracket::new(bracket_top, bracket_bottom);
            Section::new(bracket)
        });
        let _ = section.measures.entry(measure_number).or_default();

        // get or create the part group in this section.
        let part_group = section
            .part_groups
            .entry(part_group_number)
            .or_insert_with(|| {
                let brace = Brace::new(ctx.font.brace(None));
                PartGroup::new(brace)
            });
        let _ = part_group.measures.entry(measure_number).or_default();

        // get or create the part in this part group.
        let part = part_group.parts.entry(part_id.clone()).or_insert_with(|| {
            let brace = Brace::new(ctx.font.brace(None));
            Part::new(brace)
        });
        part.ensure_staves(vec![1.into()].into_iter().collect());
        for chord in part_measure.chords.iter().flat_map(|c| c.1) {
            let notes: HashSet<StaffIdx> = chord.notes.iter().map(|n| n.staff).collect();
            part.ensure_staves(notes);
        }
        part.set_visibility(ctx.layout_ctx.part_hidden_specified);
        part.hide_staves(&ctx.layout_ctx.staff.explicitly_hidden);
        part.show_staves(&ctx.layout_ctx.staff.explicitly_shown);
        part.set_distances(&ctx.layout_ctx.staff.distances, &ctx.layout.staff_distance);

        let mut clefs: BTreeMap<StaffIdx, Clef> = BTreeMap::new();
        for (idx, clef) in ctx.layout_ctx.staff.clef.iter() {
            clefs.insert(*idx, ctx.font.clef(clef));
        }
        part.set_opening_clef(&clefs);

        part.set_staff_scale(&ctx.layout_ctx.staff.staff_scaling);

        part.measures.insert(measure_number, part_measure);

        for (_, staff) in part.staves.iter_mut() {
            staff.measures.entry(measure_number).or_default();
        }
    }

    fn exit_part(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit(&mut self, _ctx: &mut WalkerCtx) {}
}

fn type_to_duration(type_str: &str) -> BaseDuration {
    match type_str {
        "maxima" => BaseDuration::Maxima,
        "longa" => BaseDuration::Longa,
        "breve" => BaseDuration::Breve,
        "whole" => BaseDuration::Whole,
        "half" => BaseDuration::Half,
        "quarter" => BaseDuration::Quarter,
        "eighth" => BaseDuration::Eighth,
        "16th" => BaseDuration::Sixteenth,
        "32nd" => BaseDuration::ThirtySecond,
        "64th" => BaseDuration::SixtyFourth,
        _ => panic!("Unknown type: {}", type_str),
    }
}

fn duration_to_notehead(dur: &BaseDuration) -> &str {
    match dur {
        BaseDuration::Maxima => "mensuralNoteheadMaximaBlack",
        BaseDuration::Longa => "mensuralNoteheadLongaWhite",
        BaseDuration::Breve => "noteheadDoubleWhole",
        BaseDuration::Whole => "noteheadWhole",
        BaseDuration::Half => "noteheadHalf",
        BaseDuration::Quarter => "noteheadBlack",
        BaseDuration::Eighth => "noteheadBlack",
        BaseDuration::Sixteenth => "noteheadBlack",
        BaseDuration::ThirtySecond => "noteheadBlack",
        BaseDuration::SixtyFourth => "noteheadBlack",
    }
}

fn duration_to_rest(dur: &BaseDuration) -> &str {
    match dur {
        BaseDuration::Maxima => "restMaxima",
        BaseDuration::Longa => "restLonga",
        BaseDuration::Breve => "restDoubleWhole",
        BaseDuration::Whole => "restWhole",
        BaseDuration::Half => "restHalf",
        BaseDuration::Quarter => "restQuarter",
        BaseDuration::Eighth => "rest8th",
        BaseDuration::Sixteenth => "rest16th",
        BaseDuration::ThirtySecond => "rest32nd",
        BaseDuration::SixtyFourth => "rest64th",
    }
}

fn duration_to_flag(dur: &BaseDuration, dir: &UpDown) -> Option<&'static str> {
    match (dur, dir) {
        (BaseDuration::Eighth, UpDown::Up) => Some("flag8thUp"),
        (BaseDuration::Eighth, UpDown::Down) => Some("flag8thDown"),

        (BaseDuration::Sixteenth, UpDown::Up) => Some("flag16thUp"),
        (BaseDuration::Sixteenth, UpDown::Down) => Some("flag16thDown"),

        (BaseDuration::ThirtySecond, UpDown::Up) => Some("flag32ndUp"),
        (BaseDuration::ThirtySecond, UpDown::Down) => Some("flag32ndDown"),

        (BaseDuration::SixtyFourth, UpDown::Up) => Some("flag64thUp"),
        (BaseDuration::SixtyFourth, UpDown::Down) => Some("flag64thDown"),

        _ => None,
    }
}
