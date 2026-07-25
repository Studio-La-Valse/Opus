use crate::layout_ctx::LayoutCtx;
use crate::score::core::clef::Clef;
use crate::score::core::key::Key;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::layout_ctx::Visibility;
use crate::utils::ReqParse;
use crate::visitor::Visitor;
use crate::xml::utils::NodeUtils;
use crate::xml::walker_ctx::WalkerCtx;
use roxmltree::Node;

pub struct LayoutContextVisitor {}

impl Visitor for LayoutContextVisitor {
    fn enter(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        ctx.layout_ctx.reset();
    }

    fn enter_work(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_defaults(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn exit_defaults(&mut self, _ctx: &mut WalkerCtx) {}

    fn enter_part_list(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_part(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        ctx.layout_ctx.reset();

        ctx.layout_ctx.part_id = _node.attribute("id").unwrap().to_string()
    }

    fn enter_measure(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        // we explicitly ignore the specified measure number because it may be either 0 or 1 based.
        ctx.layout_ctx.measure.number += 1;
        ctx.layout_ctx.measure.width = node.attribute("width").and_then(|s| s.parse::<f32>().ok());

        ctx.layout_ctx.system.margin_left = None;
        ctx.layout_ctx.system.margin_right = None;
        ctx.layout_ctx.system.distance = None;
        ctx.layout_ctx.system.distance_top = None;

        // Reset clef changes and assign the currently tracked clefs to position 0 (the beginning of the measure).
        ctx.layout_ctx.staff.clef_changes.clear();
        // setting the currently tracked clefs to the implicit clef changes at the start of the measure
        // required t correctly track clef across position.
        for (staff_idx, clef) in ctx.layout_ctx.staff.active_clef.iter() {
            ctx.layout_ctx
                .staff
                .clef_changes
                .entry(*staff_idx)
                .or_default()
                .insert(0, *clef);
        }

        ctx.layout_ctx.position = 0;

        // These 2 values are set in print. Not every measure has print, so default to false.
        ctx.layout_ctx.new_page = false;
        ctx.layout_ctx.new_system = false;
    }

    fn enter_print(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        // new-page / new-system
        ctx.layout_ctx.new_page = element
            .attribute("new-page")
            .map(|v| v == "yes")
            .unwrap_or(false);

        ctx.layout_ctx.new_system = ctx.layout_ctx.new_page
            || element
                .attribute("new-system")
                .map(|v| v == "yes")
                .unwrap_or(false)
            || ctx.layout_ctx.measure.number == 1;

        if ctx.layout_ctx.new_page {
            ctx.layout_ctx.page.page_number += 1;
        }

        if ctx.layout_ctx.new_system {
            ctx.layout_ctx.system.index += 1;

            // clear the opening clefs for the staves, set the opening to the current clefs.
            ctx.layout_ctx.staff.opening_clef.clear();
            for (idx, clef) in ctx.layout_ctx.staff.active_clef.iter() {
                ctx.layout_ctx.staff.opening_clef.insert(*idx, *clef);
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
                    ctx.layout_ctx.system.margin_left = Some(v);
                }

                if let Some(v) = right_margin {
                    ctx.layout_ctx.system.margin_right = Some(v);
                }
            }

            let system_distance = system_layout
                .children()
                .find(|n| n.has_tag_name("system-distance"))
                .and_then(|n| n.text())
                .and_then(|s| s.parse::<f32>().ok());

            if let Some(v) = system_distance {
                ctx.layout_ctx.system.distance = Some(v);
            }

            let system_distance_top = system_layout
                .children()
                .find(|n| n.has_tag_name("top-system-distance"))
                .and_then(|n| n.text())
                .and_then(|s| s.parse::<f32>().ok());

            if let Some(v) = system_distance_top {
                ctx.layout_ctx.system.distance_top = Some(v);
            }
        }

        // staff layout
        ctx.layout_ctx.staff.distances.clear();
        for staff_layout in element
            .children()
            .filter(|n| n.has_tag_name("staff-layout"))
        {
            let staff_distance = staff_layout.req_child("staff-distance").req_parse();

            let staff_number: u32 = staff_layout.req_attribute("number").req_parse();

            ctx.layout_ctx
                .staff
                .distances
                .insert(staff_number.into(), staff_distance);
        }
    }

    fn enter_attributes(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        for node in element.children() {
            if node.has_tag_name("divisions") {
                ctx.layout_ctx.divisions = node.req_parse();
            }

            if node.has_tag_name("time") {
                for node in node.children() {
                    if node.has_tag_name("beats") {
                        ctx.layout_ctx.beats = node.req_parse();
                    }

                    if node.has_tag_name("beat-type") {
                        let beat_type: u32 = node.req_parse();
                        ctx.layout_ctx.beat_type = beat_type.into();
                    }
                }
            }

            if node.has_tag_name("key") {}
        }
    }

    fn enter_clef(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        let staff: StaffIdx = element
            .get_attribute("number")
            .map(|s| s.parse::<u32>().unwrap())
            .unwrap_or(1)
            .into();
        ctx.layout_ctx.staff.number = staff;

        let sign_node = element.req_child("sign");
        let sign = sign_node.req_text();
        let line = element.get_child("line").map(|l| l.req_parse());

        let clef = Clef::from_mxml(sign, line).unwrap();
        // always track the active clef.
        ctx.layout_ctx.staff.active_clef.insert(staff, clef);

        // every clef encounter is registered as a clef change.
        let position = ctx.layout_ctx.position;
        ctx.layout_ctx
            .staff
            .clef_changes
            .entry(staff)
            .or_default()
            .insert(position, clef);

        // If this measure is the first in a system, set this clef to opening of the staff.
        // note how we use the measure number because the first time a print appears,
        // there is no new_system information available. We increment the measure number every time
        // we enter a measure, which is initialized at 0, so the first measure will always be number 1.
        if ctx.layout_ctx.measure.number == 1 && ctx.layout_ctx.position == 0 {
            ctx.layout_ctx.staff.opening_clef.insert(staff, clef);
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
                ctx.layout_ctx.staff.explicitly_hidden.insert(num);
                ctx.layout_ctx.staff.explicitly_shown.remove(&num);
            } else {
                // if not specified, hide the entire part.
                ctx.layout_ctx.part_hidden_specified = Visibility::Hidden;
            }
        }

        let restore = print_object == "yes";
        if restore {
            // if any staff is printed, set part visibility to shown.
            ctx.layout_ctx.part_hidden_specified = Visibility::Shown;

            if let Some(num) = number {
                ctx.layout_ctx.staff.explicitly_hidden.remove(&num);
                ctx.layout_ctx.staff.explicitly_shown.insert(num);
            }
        }

        let number = number.unwrap_or(1.into());
        if let Some(staff_size) = element.get_child("staff-size") {
            let mut v: f32 = staff_size.req_parse();
            v /= 100.;
            ctx.layout_ctx.staff.staff_scaling.insert(number, v);
            ctx.layout_ctx.staff.content_scaling.insert(number, v);

            if let Some(scaling) = staff_size
                .attribute("scaling")
                .map(|s| {
                    let v: f32 = s.req_parse();
                    v
                })
                .map(|s| s / 100.)
            {
                ctx.layout_ctx.staff.content_scaling.insert(number, scaling);
            }
        }
    }

    fn enter_key(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        let fifths: i8 = node.req_child("fifths").req_parse();
        let mode = node.req_child("mode").req_text();

        let key: Key = (fifths, mode).try_into().unwrap();
        ctx.layout_ctx.key = key;
    }

    fn enter_backup(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        let duration: u32 = node.req_child("duration").req_parse();
        ctx.layout_ctx.position -= duration;
    }

    fn enter_forward(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        let duration: u32 = node.req_child("duration").req_parse();
        ctx.layout_ctx.position += duration;

        validate_position(ctx.layout_ctx)
    }

    fn enter_note(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        ctx.layout_ctx.chord = false;

        if element.get_child("chord").is_some() {
            ctx.layout_ctx.chord = true;

            // we move the position backwards (by the previous note duration),
            // so that when we enter a note upstream (a subsequent callback),
            // the position is correct.
            // When exiting a note, the position is always pushed forwards
            // the duration of this note.
            ctx.layout_ctx.position -= ctx.layout_ctx.duration;
        }

        ctx.layout_ctx.grace = element.has_child("grace");

        let duration: u32 = if ctx.layout_ctx.grace {
            0
        } else {
            element.req_child("duration").req_parse()
        };

        ctx.layout_ctx.duration = duration;

        for node in element.children() {
            if node.has_tag_name("voice") {
                let voice: u32 = node.req_parse();
                ctx.layout_ctx.voice = voice.into()
            }

            if node.has_tag_name("staff") {
                let staff: u32 = node.req_parse();
                ctx.layout_ctx.staff.number = staff.into();
            }
        }
    }

    fn exit_note(&mut self, ctx: &mut WalkerCtx) {
        // We move the position forwards the duration of the note, even it is a chord.
        // When we enter a chord note, the position is moved backwards, so that
        // the position is correct upstream (a subsequent callback).
        ctx.layout_ctx.position += ctx.layout_ctx.duration;

        validate_position(ctx.layout_ctx)
    }

    fn exit_measure(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit_part(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit(&mut self, _ctx: &mut WalkerCtx) {}
}

/// Divisions are annotated per quarter note.
/// For example, if duration = 1 and divisions = 2, this is an eighth note duration.
fn validate_position(layout_ctx: &LayoutCtx) {
    let whole_beats = layout_ctx.beat_type.as_int() as f32; // eg 4. for a 3/4 measure, 8. for a 7/8 measure.
    let quarter_beats = layout_ctx.beats as f32 * (4. / whole_beats); // eg 1.5 for 3/8, 4. for 2/2.
    let divisions_in_measure = layout_ctx.divisions as f32 * quarter_beats;

    if layout_ctx.position as f32 > divisions_in_measure {
        panic!(
            "Invalid document: entered position {} in measure with {} beats of type {}, and {} divisions. Part {}, measure {}",
            layout_ctx.position,
            layout_ctx.beats,
            layout_ctx.beat_type,
            layout_ctx.divisions,
            layout_ctx.part_id,
            layout_ctx.measure.number
        );
    }
}
