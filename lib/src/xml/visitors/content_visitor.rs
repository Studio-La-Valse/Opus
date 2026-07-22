use crate::duration::BaseDuration;
use crate::score::core::pitch::Pitch;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::core::step::Step;
use crate::score::visual::note::Note;
use crate::visitor::Visitor;
use crate::visual::chord::Chord;
use crate::visual::clef::Clef;
use crate::visual::rest::Rest;
use crate::visual::stem::{BeamType, Stem, UpDown};
use crate::xml::utils::{NodeUtils, ToNumber};
use crate::xml::walker_ctx::WalkerCtx;

use roxmltree::Node;
use std::collections::HashMap;

pub struct ContentVisitor {
    pub clef_change: HashMap<StaffIdx, Clef>,
}

impl ContentVisitor {}

impl Visitor for ContentVisitor {
    fn enter(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_work(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_defaults(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn exit_defaults(&mut self, _ctx: &mut WalkerCtx) {}

    fn enter_part_list(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_part(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_measure(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_print(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_attributes(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_clef(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        let staff_idx: StaffIdx = ctx.layout_ctx.staff.number;
        let clef = ctx.layout_ctx.staff.active_clef.get(&staff_idx).unwrap();
        let smufl_clef = ctx.font.clef(clef);
        let visual_clef = Clef::new(smufl_clef);

        if ctx.layout_ctx.position > 0 {
            // Mid-measure: Anchor to a chord or rest. Store in self for now, take later.
            self.clef_change.insert(staff_idx, visual_clef);
        } else {
            // Otherwise, draw left of previous measure bar, so anchor to part measure.
            let measure_number = ctx.layout_ctx.measure.number;
            let part_id = ctx.layout_ctx.part_id.as_str();
            let previous_measure =
                ctx.visual_score
                    .locate_staff_measure_mut(part_id, &staff_idx, measure_number - 1);

            if let Some(previous_measure) = previous_measure {
                let smufl_clef = ctx.font.clef(clef);
                let drawable_clef = Clef::new(smufl_clef);
                previous_measure.prepare_clef_change = Some(drawable_clef);
            }
        }
    }

    fn enter_staff_details(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_backup(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_forward(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_note(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        let page_number = ctx.layout_ctx.page.page_number;
        let system_index = ctx.layout_ctx.system.index;
        let staff_idx = ctx.layout_ctx.staff.number;
        let measure_number = ctx.layout_ctx.measure.number;

        let position = ctx.layout_ctx.position;
        let scale = ctx
            .layout_ctx
            .staff
            .staff_scaling
            .get(&staff_idx)
            .unwrap_or(&1.);

        let voice = ctx.layout_ctx.voice;

        let part_id = ctx.layout_ctx.part_id.clone();

        // get or create the system on the page
        let system = ctx
            .visual_score
            .locate_system_mut(&page_number, &system_index)
            .unwrap();

        let is_rest = node.children().find(|n| n.tag_name().name() == "rest");

        if let Some(rest_node) = is_rest {
            let staff_measure = system
                .locate_staff_measure_mut(part_id.as_str(), staff_idx, measure_number)
                .unwrap();

            let is_measure = rest_node
                .attribute("measure")
                .map(|attr| attr == "yes")
                .unwrap_or(false);

            let mut rest = if is_measure {
                let notehead = duration_to_rest(&BaseDuration::Whole);
                let glyph = ctx.font.rest(notehead);
                Rest::new(glyph, is_measure, None, staff_idx, 4, *scale)
            } else {
                let type_str = node.req_child("type");
                let dur = type_to_duration(type_str.req_text());
                let notehead = duration_to_rest(&dur);
                let glyph = ctx.font.rest(notehead);

                let default_x: f32 = node.req_attribute("default-x").req_f32();
                Rest::new(glyph, is_measure, Some(default_x), staff_idx, 4, *scale)
            };

            for (_staff_idx, clef_change) in self.clef_change.drain() {
                rest.clef_change = Some(clef_change);
            }

            staff_measure.rests.push(rest);
        } else {
            let part_id = part_id.as_str();
            let part_measure = system
                .locate_part_measure_mut(part_id, measure_number)
                .unwrap();

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

            let clef = ctx.layout_ctx.staff.active_clef(&staff_idx, &position);
            let staff_line = clef.line_index_at_pitch(&pitch);
            let type_str = node.req_child("type");

            let dur = type_to_duration(type_str.req_text());
            let notehead = duration_to_notehead(&dur);

            let glyph = ctx.font.notehead(notehead);

            let note = Note::new(glyph, default_x, staff_idx, staff_line, *scale);
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
                    .get_or_insert_with(|| Stem::new(dir, dur, staff_idx, *scale, default_y));

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

            for (staff_idx, clef_change) in self.clef_change.drain() {
                chord.clef_change.insert(staff_idx, clef_change);
            }

            chord.notes.push(note);
        }
    }

    fn exit_note(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit_measure(&mut self, _ctx: &mut WalkerCtx) {}

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
