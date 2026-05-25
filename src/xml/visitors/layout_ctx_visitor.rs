use roxmltree::Node;

use crate::score::layout_ctx::{LayoutCtx, Visibility};
use crate::xml::visitor::Visitor;

pub struct LayoutContextVisitor<'a> {
    pub layout_ctx: &'a mut LayoutCtx,
}

impl<'a> LayoutContextVisitor<'a> {
    fn reset(&mut self) {
        self.layout_ctx.system.index = 0;

        self.layout_ctx.page.page_number = 1;
        self.layout_ctx.staff.number = 1;

        self.layout_ctx.system.margin_left = None;
        self.layout_ctx.system.margin_right = None;
        self.layout_ctx.system.distance = None;
        self.layout_ctx.system.distance_top = None;

        self.layout_ctx.staff.distances.clear();

        self.layout_ctx.part_hidden_specified = Visibility::Auto;
        self.layout_ctx.staff.explicitly_hidden.clear();
        self.layout_ctx.staff.explicitly_shown.clear();
    }
}

impl<'a> Visitor for LayoutContextVisitor<'a> {
    fn enter(&mut self, _node: &Node) {
        self.reset()
    }

    fn enter_work(&mut self, _node: &Node) {}

    fn enter_defaults(&mut self, _node: &Node) {}

    fn exit_defaults(&mut self) {}

    fn enter_part_list(&mut self, _node: &Node) {}

    fn enter_part(&mut self, _node: &Node) {
        self.reset();

        self.layout_ctx.part_id = _node.attribute("id").unwrap().to_string()
    }

    fn enter_measure(&mut self, _node: &Node) {
        self.layout_ctx.measure.number = _node
            .attribute("number")
            .expect("measure missing @number")
            .parse::<u32>()
            .expect("measure number was not an integer");

        self.layout_ctx.measure.width =
            _node.attribute("width").and_then(|s| s.parse::<f32>().ok());

        self.layout_ctx.system.margin_left = None;
        self.layout_ctx.system.margin_right = None;
        self.layout_ctx.system.distance = None;
        self.layout_ctx.system.distance_top = None;
    }

    fn enter_print(&mut self, element: &Node) {
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
            self.layout_ctx.page.page_number += 1;
        }

        if new_system {
            self.layout_ctx.system.index += 1;
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
                    self.layout_ctx.system.margin_left = Some(v);
                }

                if let Some(v) = right_margin {
                    self.layout_ctx.system.margin_right = Some(v);
                }
            }

            let system_distance = system_layout
                .children()
                .find(|n| n.has_tag_name("system-distance"))
                .and_then(|n| n.text())
                .and_then(|s| s.parse::<f32>().ok());

            if let Some(v) = system_distance {
                self.layout_ctx.system.distance = Some(v);
            }

            let system_distance_top = system_layout
                .children()
                .find(|n| n.has_tag_name("top-system-distance"))
                .and_then(|n| n.text())
                .and_then(|s| s.parse::<f32>().ok());

            if let Some(v) = system_distance_top {
                self.layout_ctx.system.distance_top = Some(v);
            }
        }

        // staff-layout
        self.layout_ctx.staff.distances.clear();

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

        self.layout_ctx
            .staff
            .distances
            .insert(staff_number, staff_distance_value);
    }

    fn enter_attributes(&mut self, element: &Node) {
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
                    self.layout_ctx.staff.explicitly_hidden.insert(num);
                    self.layout_ctx.staff.explicitly_shown.remove(&num);
                } else {
                    self.layout_ctx.staff.visibility = Visibility::Hidden;
                }
            }

            let restore = print_object == "yes";
            if restore {
                self.layout_ctx.staff.visibility = Visibility::Shown;

                if let Some(num) = staff_details
                    .attribute("number")
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    self.layout_ctx.staff.explicitly_hidden.remove(&num);
                    self.layout_ctx.staff.explicitly_shown.insert(num);
                }
            }
        }
    }

    fn enter_note(&mut self, element: &Node) {
        if let Some(staff) = element
            .children()
            .find(|n| n.has_tag_name("staff"))
            .and_then(|n| n.text())
            .and_then(|s| s.parse::<u32>().ok())
        {
            self.layout_ctx.staff.number = staff;
        }
    }

    fn exit_measure(&mut self) {}

    fn exit_part(&mut self) {}

    fn exit(&mut self) {}
}
