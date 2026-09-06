use crate::musicxml::utils::NodeUtils;
use crate::musicxml::utils::ReqParse;
use crate::musicxml::visitor::Visitor;
use crate::musicxml::walker_ctx::WalkerCtx;
use crate::score::core::clef::Clef;
use crate::score::core::key::{Key, Mode};
use crate::score::core::staff_idx::StaffIdx;
use crate::score::walk_cursor::Visibility;
use roxmltree::Node;

pub struct WalkCursorVisitor {}

impl<'a> Visitor<WalkerCtx<'a>> for WalkCursorVisitor {
    fn enter(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        ctx.cursor.reset();
    }

    fn enter_part(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        ctx.cursor.reset();

        ctx.cursor.part_id = _node.attribute("id").unwrap().to_string()
    }

    fn enter_measure(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        // we explicitly ignore the specified measure number because it may be either 0 or 1 based.
        ctx.cursor.measure.number += 1;
        ctx.cursor.measure.width = node.attribute("width").and_then(|s| s.parse::<f32>().ok());

        ctx.cursor.system.margin_left = None;
        ctx.cursor.system.margin_right = None;
        ctx.cursor.system.distance = None;
        ctx.cursor.system.distance_top = None;

        // Reset clef changes and assign the currently tracked clefs to position 0 (the beginning of the measure).
        ctx.cursor.staff.clef_changes.clear();
        // setting the currently tracked clefs to the implicit clef changes at the start of the measure
        // required t correctly track clef across position.
        for (staff_idx, clef) in ctx.cursor.staff.active_clef.iter() {
            ctx.cursor
                .staff
                .clef_changes
                .entry(*staff_idx)
                .or_default()
                .insert(0, *clef);
        }

        ctx.cursor.begin_measure();

        // These 2 values are set in print. Not every measure has print, so default to false.
        ctx.cursor.new_page = false;
        ctx.cursor.new_system = false;
    }

    fn enter_print(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        // new-page / new-system
        ctx.cursor.new_page = element
            .attribute("new-page")
            .map(|v| v == "yes")
            .unwrap_or(false);

        ctx.cursor.new_system = ctx.cursor.new_page
            || element
                .attribute("new-system")
                .map(|v| v == "yes")
                .unwrap_or(false)
            || ctx.cursor.measure.number == 1;

        if ctx.cursor.new_page {
            ctx.cursor.page.page_number += 1;
        }

        if ctx.cursor.new_system {
            ctx.cursor.system.index += 1;

            // clear the opening clefs for the staves, set the opening to the current clefs.
            ctx.cursor.staff.opening_clef.clear();
            for (idx, clef) in ctx.cursor.staff.active_clef.iter() {
                ctx.cursor.staff.opening_clef.insert(*idx, *clef);
            }
        }

        // system-layout
        if let Some(system_layout) = element.children().find(|n| n.has_tag_name("system-layout")) {
            if let Some(system_margins) = system_layout
                .children()
                .find(|n| n.has_tag_name("system-margins"))
            {
                let left_margin = system_margins
                    .children()
                    .find(|n| n.has_tag_name("left-margin"))
                    .and_then(|n| n.text())
                    .and_then(|s| s.parse::<f32>().ok());

                let right_margin = system_margins
                    .children()
                    .find(|n| n.has_tag_name("right-margin"))
                    .and_then(|n| n.text())
                    .and_then(|s| s.parse::<f32>().ok());

                if let Some(v) = left_margin {
                    ctx.cursor.system.margin_left = Some(v);
                }

                if let Some(v) = right_margin {
                    ctx.cursor.system.margin_right = Some(v);
                }
            }

            let system_distance = system_layout
                .children()
                .find(|n| n.has_tag_name("system-distance"))
                .and_then(|n| n.text())
                .and_then(|s| s.parse::<f32>().ok());

            if let Some(v) = system_distance {
                ctx.cursor.system.distance = Some(v);
            }

            let system_distance_top = system_layout
                .children()
                .find(|n| n.has_tag_name("top-system-distance"))
                .and_then(|n| n.text())
                .and_then(|s| s.parse::<f32>().ok());

            if let Some(v) = system_distance_top {
                ctx.cursor.system.distance_top = Some(v);
            }
        }

        // staff layout
        ctx.cursor.staff.distances.clear();
        for staff_layout in element
            .children()
            .filter(|n| n.has_tag_name("staff-layout"))
        {
            let staff_distance = staff_layout.req_child("staff-distance").req_parse();

            let staff_number: u32 = staff_layout.req_attribute("number").req_parse();

            ctx.cursor
                .staff
                .distances
                .insert(staff_number.into(), staff_distance);
        }
    }

    fn enter_attributes(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        for node in element.children() {
            if node.has_tag_name("divisions") {
                ctx.cursor.set_divisions(node.req_parse());
            }

            if node.has_tag_name("time") {
                for node in node.children() {
                    if node.has_tag_name("beats") {
                        ctx.cursor.set_beats(node.req_parse());
                    }

                    if node.has_tag_name("beat-type") {
                        let beat_type: u32 = node.req_parse();
                        ctx.cursor.set_beat_type(beat_type.into());
                    }
                }
            }
        }
    }

    fn enter_clef(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        let staff: StaffIdx = element
            .get_attribute("number")
            .map(|s| s.parse::<u32>().unwrap())
            .unwrap_or(1)
            .into();
        ctx.cursor.staff.number = staff;

        let sign_node = element.req_child("sign");
        let sign = sign_node.req_text();
        let line = element.get_child("line").map(|l| l.req_parse());

        let clef = Clef::from_mxml(sign, line).unwrap();
        // always track the active clef.
        ctx.cursor.staff.active_clef.insert(staff, clef);

        // every clef encounter is registered as a clef change.
        let position = ctx.cursor.position;
        ctx.cursor
            .staff
            .clef_changes
            .entry(staff)
            .or_default()
            .insert(position, clef);

        // If this measure is the first in a system, set this clef to opening of the staff.
        // note how we use the measure number because the first time a print appears,
        // there is no new_system information available. We increment the measure number every time
        // we enter a measure, which is initialized at 0, so the first measure will always be number 1.
        if ctx.cursor.measure.number == 1 && ctx.cursor.position == 0 {
            ctx.cursor.staff.opening_clef.insert(staff, clef);
        }
    }

    fn enter_staff_details(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        let print_object = element.attribute("print-object").unwrap_or("yes");

        let number: Option<StaffIdx> = element.attribute("number").map(|s| {
            let s: u32 = s.req_parse();
            s.into()
        });

        let is_hidden = print_object == "no";
        if is_hidden {
            if let Some(num) = number {
                // if a stuff number is specified, hide the staff.
                ctx.cursor.staff.explicitly_hidden.insert(num);
                ctx.cursor.staff.explicitly_shown.remove(&num);
            } else {
                // if not specified, hide the entire part.
                ctx.cursor.part_hidden_specified = Visibility::Hidden;
            }
        }

        let restore = print_object == "yes";
        if restore {
            // if any staff is printed, set part visibility to shown.
            ctx.cursor.part_hidden_specified = Visibility::Shown;

            if let Some(num) = number {
                ctx.cursor.staff.explicitly_hidden.remove(&num);
                ctx.cursor.staff.explicitly_shown.insert(num);
            }
        }

        let number = number.unwrap_or(1.into());
        if let Some(staff_lines) = element.get_child("staff-lines") {
            let lines: usize = staff_lines.req_parse();
            ctx.cursor.staff.lines.insert(number, lines);
        }

        if let Some(staff_size) = element.get_child("staff-size") {
            let mut v: f32 = staff_size.req_parse();
            v /= 100.;
            ctx.cursor.staff.staff_scaling.insert(number, v);
            ctx.cursor.staff.content_scaling.insert(number, v);

            if let Some(scaling) = staff_size
                .attribute("scaling")
                .map(|s| {
                    let v: f32 = s.req_parse();
                    v
                })
                .map(|s| s / 100.)
            {
                ctx.cursor.staff.content_scaling.insert(number, scaling);
            }
        }
    }

    fn enter_key(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        let fifths: i8 = node.req_child("fifths").req_parse();

        // `<mode>` is optional, and a document that does write one may spell it
        // `none` or as a church mode. None of that changes the printed
        // signature -- `<fifths>` already is the accidental count -- so anything
        // but an outright "minor" is read as major. See `Key::from_mxml`.
        let mode = node
            .get_child("mode")
            .and_then(|n| n.text())
            .and_then(|text| Mode::try_from(text.trim()).ok())
            .unwrap_or(Mode::Major);

        ctx.cursor.key = Key::from_mxml(fifths, mode);
    }

    fn enter_backup(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        let duration: u32 = node.req_child("duration").req_parse();
        ctx.cursor
            .apply_backup(duration)
            .expect("backup duration exceeds current position");
    }

    fn enter_forward(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        let duration: u32 = node.req_child("duration").req_parse();
        ctx.cursor
            .apply_forward(duration)
            .expect("forward duration overflowed position");
    }

    fn enter_note(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        // First thing, so every visitor chained after this one reads the id of
        // the note it is currently being handed.
        ctx.cursor.advance_note_id();

        let is_chord = element.get_child("chord").is_some();
        let is_grace = element.has_child("grace");
        let is_cue = element.has_child("cue");

        let duration: u32 = if is_grace {
            0
        } else {
            element.req_child("duration").req_parse()
        };

        ctx.cursor
            .enter_note(duration, is_chord, is_grace, is_cue)
            .expect("chord note duration exceeds current position");

        for node in element.children() {
            if node.has_tag_name("voice") {
                let voice: u32 = node.req_parse();
                ctx.cursor.voice = voice.into()
            }

            if node.has_tag_name("staff") {
                let staff: u32 = node.req_parse();
                ctx.cursor.staff.number = staff.into();
            }
        }
    }

    fn exit_note(&mut self, ctx: &mut WalkerCtx) {
        // We move the position forwards the duration of the note, even it is a chord.
        // When we enter a chord note, the position is moved backwards, so that
        // the position is correct upstream (a subsequent callback).
        ctx.cursor
            .exit_note()
            .expect("note duration overflowed position");
    }
}
