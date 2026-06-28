use crate::score::core::clef::Clef;
use crate::score::layout_ctx::Visibility;
use crate::utils::xml::{N, ToNumber};
use crate::visitor::Visitor;
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

    fn enter_measure(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        ctx.layout_ctx.measure.number = _node
            .attribute("number")
            .expect("measure missing @number")
            .parse::<u32>()
            .expect("measure number was not an integer");

        ctx.layout_ctx.measure.width = _node.attribute("width").and_then(|s| s.parse::<f32>().ok());

        ctx.layout_ctx.system.margin_left = None;
        ctx.layout_ctx.system.margin_right = None;
        ctx.layout_ctx.system.distance = None;
        ctx.layout_ctx.system.distance_top = None;

        ctx.layout_ctx.position = 0;
    }

    fn enter_print(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        // new-page / new-system
        let new_page = element
            .attribute("new-page")
            .map(|v| v == "yes")
            .unwrap_or(false);

        let new_system = new_page
            || element
                .attribute("new-system")
                .map(|v| v == "yes")
                .unwrap_or(false);

        if new_page {
            ctx.layout_ctx.page.page_number += 1;
        }

        if new_system {
            ctx.layout_ctx.system.index += 1;
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

        ctx.layout_ctx.staff.distances.clear();

        for staff_layout in element
            .children()
            .filter(|n| n.has_tag_name("staff-layout"))
        {
            let staff_distance = staff_layout.req_child("staff-distance").req_f32();

            let staff_number = staff_layout.req_attribute("number").req_u32();

            ctx.layout_ctx
                .staff
                .distances
                .insert(staff_number, staff_distance);
        }
    }

    fn enter_attributes(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        for node in element.children() {
            if node.has_tag_name("divisions") {
                ctx.layout_ctx.divisions = node.req_u32();
            }

            if node.has_tag_name("time") {
                for node in node.children() {
                    if node.has_tag_name("beats") {
                        ctx.layout_ctx.beats = node.req_u32();
                    }

                    if node.has_tag_name("beat-type") {
                        ctx.layout_ctx.beat_type = node.req_u32();
                    }
                }
            }

            if node.has_tag_name("staff-details") {
                self.enter_staff_details(&node, ctx);
            }

            if node.has_tag_name("clef") {
                self.enter_clef(&node, ctx);
            }
        }
    }

    fn enter_clef(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        let staff = element
            .get_attribute("number")
            .map(|s| s.parse::<u32>().unwrap())
            .unwrap_or(1);
        let sign_node = element.req_child("sign");
        let sign = sign_node.req_text();
        let line = element.get_child("line").map(|l| l.req_i32());

        let clef = Clef::parse(sign, line);
        ctx.layout_ctx.clef.insert(staff, clef);
    }

    fn enter_staff_details(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        let print_object = element.attribute("print-object").unwrap_or("yes");

        let is_hidden = print_object == "no";

        if is_hidden {
            if let Some(num) = element
                .attribute("number")
                .and_then(|s| s.parse::<u32>().ok())
            {
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

            if let Some(num) = element
                .attribute("number")
                .and_then(|s| s.parse::<u32>().ok())
            {
                ctx.layout_ctx.staff.explicitly_hidden.remove(&num);
                ctx.layout_ctx.staff.explicitly_shown.insert(num);
            }
        }
    }

    fn enter_backup(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        let duration = node.req_child("duration").req_u32();
        ctx.layout_ctx.position -= duration;
    }

    fn enter_forward(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        let duration = node.req_child("duration").req_u32();
        ctx.layout_ctx.position += duration;

        if ctx.layout_ctx.position > ctx.layout_ctx.divisions * ctx.layout_ctx.beats {
            panic!(
                "Invalid document: entered position {} in measure with {} beats and {} divisions. Part {}, measure {}",
                ctx.layout_ctx.position,
                ctx.layout_ctx.beats,
                ctx.layout_ctx.divisions,
                ctx.layout_ctx.part_id,
                ctx.layout_ctx.measure.number
            );
        }
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

        let duration = element.req_child("duration").req_u32();
        ctx.layout_ctx.duration = duration;

        for node in element.children() {
            if node.has_tag_name("voice") {
                ctx.layout_ctx.voice = node.req_u32()
            }

            if node.has_tag_name("staff") {
                ctx.layout_ctx.staff.number = node.req_u32();
            }
        }
    }

    fn exit_note(&mut self, ctx: &mut WalkerCtx) {
        // We move the position forwards the duration of the note, even it is a chord.
        // When we enter a chord note, the position is moved backwards, so that
        // the position is correct upstream (a subsequent callback).
        ctx.layout_ctx.position += ctx.layout_ctx.duration;

        if ctx.layout_ctx.position > ctx.layout_ctx.divisions * ctx.layout_ctx.beats {
            panic!(
                "Invalid document: entered position {} in measure with {} beats and {} divisions. Part {}, measure {}",
                ctx.layout_ctx.position,
                ctx.layout_ctx.beats,
                ctx.layout_ctx.divisions,
                ctx.layout_ctx.part_id,
                ctx.layout_ctx.measure.number
            );
        }
    }

    fn exit_measure(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit_part(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit(&mut self, _ctx: &mut WalkerCtx) {}
}
