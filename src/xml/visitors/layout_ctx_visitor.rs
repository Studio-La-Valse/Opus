use roxmltree::Node;

use crate::score::layout_ctx::{LayoutCtx, Visibility};

pub struct LayoutContextVisitor {

}

impl LayoutContextVisitor {

    pub fn enter(&mut self, _node: &Node, layout_ctx: &mut LayoutCtx) {
        layout_ctx.reset();
    }

    pub fn enter_part(&mut self, _node: &Node, layout_ctx: &mut LayoutCtx) {
        layout_ctx.reset();

        layout_ctx.part_id = _node.attribute("id").unwrap().to_string()
    }

    pub  fn enter_measure(&mut self, _node: &Node, layout_ctx: &mut LayoutCtx) {
        layout_ctx.measure.number = _node
            .attribute("number")
            .expect("measure missing @number")
            .parse::<u32>()
            .expect("measure number was not an integer");

        layout_ctx.measure.width =
            _node.attribute("width").and_then(|s| s.parse::<f32>().ok());

        layout_ctx.system.margin_left = None;
        layout_ctx.system.margin_right = None;
        layout_ctx.system.distance = None;
        layout_ctx.system.distance_top = None;
    }

    pub fn enter_print(&mut self, element: &Node, layout_ctx: &mut LayoutCtx) {
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
            layout_ctx.page.page_number += 1;
        }

        if new_system {
            layout_ctx.system.index += 1;
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
                    layout_ctx.system.margin_left = Some(v);
                }

                if let Some(v) = right_margin {
                    layout_ctx.system.margin_right = Some(v);
                }
            }

            let system_distance = system_layout
                .children()
                .find(|n| n.has_tag_name("system-distance"))
                .and_then(|n| n.text())
                .and_then(|s| s.parse::<f32>().ok());

            if let Some(v) = system_distance {
                layout_ctx.system.distance = Some(v);
            }

            let system_distance_top = system_layout
                .children()
                .find(|n| n.has_tag_name("top-system-distance"))
                .and_then(|n| n.text())
                .and_then(|s| s.parse::<f32>().ok());

            if let Some(v) = system_distance_top {
                layout_ctx.system.distance_top = Some(v);
            }
        }

        layout_ctx.staff.distances.clear();

        let staff_layout = match element.children().find(|n| n.has_tag_name("staff-layout")) {
            Some(n) => n,
            None => return,
        };

        let staff_distance = match staff_layout
            .children()
            .find(|n| n.has_tag_name("staff-distance"))
        {
            Some(n) => n,
            None => return,
        };

        let staff_number = staff_layout
            .attribute("number")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1);

        let staff_distance_value = staff_distance
            .text()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);

        layout_ctx
            .staff
            .distances
            .insert(staff_number, staff_distance_value);
    }

    pub fn enter_attributes(&mut self, element: &Node, layout_ctx: &mut LayoutCtx) {
        for staff_details in element
            .children()
            .filter(|n| n.has_tag_name("staff-details"))
        {
            let print_object = staff_details.attribute("print-object").unwrap_or("yes");

            let is_hidden = print_object == "no";

            if is_hidden {
                if let Some(num) = staff_details
                    .attribute("number")
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    layout_ctx.staff.explicitly_hidden.insert(num);
                    layout_ctx.staff.explicitly_shown.remove(&num);
                } else {
                    layout_ctx.staff.visibility = Visibility::Hidden;
                }
            }

            let restore = print_object == "yes";
            if restore {
                layout_ctx.staff.visibility = Visibility::Shown;

                if let Some(num) = staff_details
                    .attribute("number")
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    layout_ctx.staff.explicitly_hidden.remove(&num);
                    layout_ctx.staff.explicitly_shown.insert(num);
                }
            }
        }
    }

    pub fn enter_note(&mut self, element: &Node, layout_ctx: &mut LayoutCtx) {
        if let Some(staff) = element
            .children()
            .find(|n| n.has_tag_name("staff"))
            .and_then(|n| n.text())
            .and_then(|s| s.parse::<u32>().ok())
        {
            layout_ctx.staff.number = staff;
        }
    }
}
