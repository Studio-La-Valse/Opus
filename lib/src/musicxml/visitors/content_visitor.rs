use crate::musicxml::utils::NodeUtils;
use crate::musicxml::visitor::Visitor;
use crate::musicxml::walker_ctx::WalkerCtx;
use crate::score::core::clef::Clef as ClefCore;
use crate::score::core::duration_base::BaseDuration;
use crate::score::core::pitch::Pitch;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::core::step::Step;
use crate::score::visual::chord::Chord;
use crate::score::visual::note::Note;
use crate::score::visual::note_scale::NoteScale;
use crate::score::visual::rest::Rest;
use crate::score::visual::stem::{BeamType, Stem, UpDown};

use crate::musicxml::utils::ReqParse;
use crate::score::core::accidental::Accidental as AccidentalCore;
use crate::score::core::key::Key;
use crate::score::core::time_signature::TimeSignature as TimeSignatureCore;
use crate::score::visual::accidental::Accidental as DrawableAccidental;
use crate::score::visual::flag::Flag as DrawableFlag;
use crate::score::visual::time_signature::TimeSignature as VisualTimeSignature;

use crate::score::visual::part::Part;
use crate::smufl::smufl_font::SmuflFont;
use roxmltree::Node;
use std::collections::BTreeMap;

pub struct ContentVisitor {}

impl ContentVisitor {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ContentVisitor {
    fn default() -> Self {
        Self::new()
    }
}

/// The size the note or rest under the cursor is drawn at: the scaling its staff
/// applies to content, plus whether it is a normal, grace or cue note.
///
/// The two stay unresolved on purpose. The grace/cue reduction comes from
/// `<note-size>`, which a caller can override, and this walk is cached and
/// re-arranged for whatever layout a later render asks for -- a factor
/// multiplied in here would be stuck at whatever the first render happened to
/// ask for. See [`NoteScale`].
fn note_scale(ctx: &WalkerCtx) -> NoteScale {
    let staff_idx = ctx.cursor.staff.number;
    let content_scale = *ctx
        .cursor
        .staff
        .content_scaling
        .get(&staff_idx)
        .unwrap_or(&1.0);

    NoteScale::new(content_scale, ctx.cursor.note_kind())
}

impl ContentVisitor {
    fn handle_rest(&mut self, node: &Node, rest_node: &Node, ctx: &mut WalkerCtx) {
        let note_id = ctx.cursor.note_id;
        let staff_idx = ctx.cursor.staff.number;
        let measure_number = ctx.cursor.measure.number;
        let part_id = ctx.cursor.part_id.as_str();
        let size = note_scale(ctx);

        // A rest is centred on the middle of its own staff, which is the single
        // line of a one-line percussion staff, not the middle of a notional
        // five-line one.
        let center_line = Rest::center_line(ctx.cursor.staff.lines(&staff_idx));

        let staff_measure = ctx
            .visual_score
            .locate_staff_measure_mut(part_id, &staff_idx, measure_number)
            .expect("Staff measure missing in system");

        let is_measure = rest_node.attribute("measure") == Some("yes");

        let dots: u8 = node.get_children("dot").len().try_into().unwrap();

        let rest = if is_measure {
            let glyph = ctx.font.rest(BaseDuration::Whole.rest_glyph());
            Rest::new(note_id, glyph, None, staff_idx, center_line, size, dots)
        } else {
            let dur: BaseDuration = node.req_child("type").req_text().try_into().unwrap();
            let glyph = ctx.font.rest(dur.rest_glyph());
            let default_x: f32 = node.req_attribute("default-x").req_parse();
            Rest::new(
                note_id,
                glyph,
                Some(default_x),
                staff_idx,
                center_line,
                size,
                dots,
            )
        };

        staff_measure.rests.push(rest);
    }

    fn handle_note(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        let default_x: f32 = match node.get_attribute("default-x").and_then(|s| s.parse().ok()) {
            Some(x) => x,
            None => return, // Ignore unpositioned notes
        };

        let pitch_node = match node.get_child("pitch") {
            Some(n) => n,
            None => return,
        };

        let system_index = ctx.cursor.system.index;
        let staff_idx = ctx.cursor.staff.number;
        let measure_number = ctx.cursor.measure.number;
        let position = ctx.cursor.position;
        let voice = ctx.cursor.voice;
        let part_id = ctx.cursor.part_id.as_str();
        let size = note_scale(ctx);

        // Parse Pitch
        let step = pitch_node.req_child("step");
        let step_str = step.req_text();
        let alter = pitch_node.get_child("alter").map_or(0, |n| n.req_parse());
        let octave = pitch_node.req_child("octave").req_parse();
        let pitch = Pitch {
            step: Step::parse(step_str, alter),
            octave,
        };

        // Determine Notehead & Visual Position
        let clef = ctx.cursor.staff.active_clef(&staff_idx, &position);
        let staff_line = clef.line_index_at_pitch(&pitch);

        let dur: BaseDuration = node.req_child("type").req_text().try_into().unwrap();
        let notehead = dur.notehead_glyph();
        let glyph = ctx.font.notehead(notehead);
        let dots: u8 = node.get_children("dot").len().try_into().unwrap();
        let mut note = Note::new(
            ctx.cursor.note_id,
            glyph,
            default_x,
            staff_idx,
            staff_line,
            size,
            dots,
        );

        // Locate Measure & Voice Chords
        let system = ctx
            .visual_score
            .locate_system_mut(&system_index)
            .expect("System missing in layout context");

        let part_measure = system
            .locate_part_measure_mut(part_id, measure_number)
            .expect("Part measure missing");

        let is_chord = node.get_child("chord").is_some();
        let chords = part_measure.chords.entry(voice).or_default();
        if !is_chord {
            chords.push(Chord::default());
        }

        let chord = chords.last_mut().expect("Chord entry should exist");
        chord.grace = ctx.cursor.grace;

        // Parse Stem & Beams.
        //
        // `<stem>` also admits `none` (an explicitly stemless note) and `double`
        // (a stem in both directions, for a shared notehead). Neither names a
        // direction, so both are handled the way a note with no `<stem>` at all
        // is: no stem, and therefore no flag or beam hanging off one.
        if let Some(stem_node) = node.children().find(|n| n.tag_name().name() == "stem")
            && let Ok(dir) = UpDown::try_from(stem_node.req_text().trim())
        {
            let default_y = stem_node.attribute("default-y").map(|a| a.req_parse());

            let stem = chord
                .stem
                .get_or_insert_with(|| Stem::new(dir, dur, staff_idx, size, default_y));

            let beams: Vec<_> = node
                .children()
                .filter(|n| n.tag_name().name() == "beam")
                .collect();
            if beams.is_empty() && stem.beams.is_empty() {
                if let Some(flag_name) = dur.flag_glyph(&dir) {
                    stem.flag = Some(DrawableFlag::new(ctx.font.flag(flag_name, &dir), size));
                }
            } else {
                for beam in beams {
                    let number = beam.req_attribute("number").req_parse();
                    let beam_type: BeamType = beam.req_text().into();
                    stem.beams.insert(number, beam_type);
                }
            }
        }

        if let Some(accidental) = node.get_child("accidental") {
            let accidental = accidental.req_text();
            let accidental: AccidentalCore = accidental.try_into().unwrap();
            let accidental = ctx.font.accidental(accidental);
            let accidental = DrawableAccidental::new(accidental);
            note.accidental = Some(accidental)
        }

        chord.notes.push(note);
    }

    /// Populates key signature accidentals at the start of a measure for all staves in a part.
    fn populate_key_signature(
        &self,
        part: &mut Part,
        measure_number: u32,
        key: Key,
        active_clef: &BTreeMap<StaffIdx, ClefCore>,
        font: &SmuflFont,
    ) {
        let n_accidentals = key.accidentals();

        if n_accidentals == 0 {
            return;
        }

        for (idx, staff) in part.staves.iter_mut() {
            let Some(staff_measure) = staff.measures.get_mut(&measure_number) else {
                continue;
            };
            let Some(clef) = active_clef.get(idx) else {
                continue;
            };

            let lines = if n_accidentals > 0 {
                clef.sharp_lines()
            } else {
                clef.flat_lines()
            };

            for line in lines.iter().take(n_accidentals.unsigned_abs() as usize) {
                let accidental_type = if n_accidentals > 0 {
                    AccidentalCore::Sharp
                } else {
                    AccidentalCore::Flat
                };

                let smufl = font.accidental(accidental_type);
                let drawable = DrawableAccidental::new(smufl);
                staff_measure
                    .key_signature_start
                    .accidentals
                    .push((*line, drawable));
            }
        }
    }
}

impl<'a> Visitor<WalkerCtx<'a>> for ContentVisitor {
    fn enter_attributes(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        if node.has_child("time") {
            let page_number = &ctx.cursor.page.page_number;
            let system_index = &ctx.cursor.system.index;
            let measure_number = ctx.cursor.measure.number;

            let part_id = &ctx.cursor.part_id.clone();
            let assignment = ctx.layout.lookup(part_id).unwrap();
            let section_number = &assignment.section;
            let part_group_number = &assignment.part_group;

            let page = ctx.visual_score.pages.get_mut(page_number).unwrap();
            let system = page.systems.get_mut(system_index).unwrap();
            let section = system.sections.get_mut(section_number).unwrap();
            let part_group = section.part_groups.get_mut(part_group_number).unwrap();
            let part = part_group.parts.get_mut(part_id).unwrap();

            for staff_measure in part.staff_measures_mut(&measure_number) {
                let time_signature = TimeSignatureCore {
                    time: ctx.cursor.beats,
                    base: ctx.cursor.beat_type,
                };
                let (num, denom) = ctx.font.time_signature(time_signature);
                let visual = VisualTimeSignature::new(num, denom);
                staff_measure.time_signature_start = Some(visual);
            }

            // this measure is the first measure in a system, in the previous measure, prepare the change.
            let is_new_system = ctx.cursor.new_system;
            if measure_number > 1 && is_new_system {
                let prev_measure_number = measure_number - 1;
                for staff_measure in ctx
                    .visual_score
                    .locate_staff_measures_mut(part_id, prev_measure_number)
                {
                    let time_signature = TimeSignatureCore {
                        time: ctx.cursor.beats,
                        base: ctx.cursor.beat_type,
                    };
                    let (num, denom) = ctx.font.time_signature(time_signature);
                    let visual = VisualTimeSignature::new(num, denom);
                    staff_measure.time_signature_end = Some(visual);
                }
            }
        }
    }

    fn enter_key(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        if ctx.cursor.new_system {
            // handled in exit_measure() for new systems
            return;
        }

        let page_number = ctx.cursor.page.page_number;
        let system_index = ctx.cursor.system.index;
        let part_id = ctx.cursor.part_id.as_str();
        let measure_number = ctx.cursor.measure.number;

        // Extract copyable/borrowable fields up front
        let key = ctx.cursor.key;

        let page = ctx.visual_score.pages.get_mut(&page_number).unwrap();
        let system = page.systems.get_mut(&system_index).unwrap();
        let part = system.locate_part_mut(part_id).unwrap();

        self.populate_key_signature(
            part,
            measure_number,
            key,
            &ctx.cursor.staff.active_clef,
            ctx.font,
        );
    }

    fn enter_note(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        let is_rest = node.children().find(|n| n.tag_name().name() == "rest");

        if let Some(rest_node) = is_rest {
            self.handle_rest(node, &rest_node, ctx);
        } else {
            self.handle_note(node, ctx);
        }
    }

    /// The key signature a new system opens with, which -- unlike one written
    /// mid-score, handled in [`enter_key`](Self::enter_key) -- has no `<key>` of
    /// its own to hang off: a system repeats the key it is already in.
    fn exit_measure(&mut self, ctx: &mut WalkerCtx) {
        if !ctx.cursor.new_system {
            return;
        }

        let page_number = ctx.cursor.page.page_number;
        let system_index = ctx.cursor.system.index;
        let measure_number = ctx.cursor.measure.number;
        let key = ctx.cursor.key;

        let part_id = ctx.cursor.part_id.clone();
        let assignment = ctx.layout.lookup(&part_id).unwrap();

        // get or create page -> system -> section -> part group -> part
        let part = ctx.visual_score.locate_or_create_part(
            page_number,
            system_index,
            &assignment,
            &part_id,
        );

        self.populate_key_signature(
            part,
            measure_number,
            key,
            &ctx.cursor.staff.active_clef,
            ctx.font,
        );
    }
}
