use crate::score::layout::{Layout, PageMargins, Part, UserLayout};
use crate::xml::utils::xml::N;
use roxmltree::Node;


fn brace_type(node: &Node) -> Option<String> {
    node.children()
        .find(|n| n.has_tag_name("group-symbol"))
        .and_then(|n| n.text())
        .map(|s| s.to_string())
}

pub struct LayoutVisitor {

}

impl LayoutVisitor {
    pub fn enter_work(&mut self, element: &Node, layout: &mut Layout) {
        layout.work_title = element.req_element("work-title").req_text().into();
    }

    pub fn enter_defaults(&mut self, element: &Node, layout: &mut Layout) {
        let scaling = element.req_child("scaling");
        layout.defaults.scaling_millimeters = scaling.req_child("millimeters").req_f32();
        layout.defaults.scaling_tenths = scaling.req_child("tenths").req_f32();

        let page_layout = element.req_child("page-layout");
        layout.defaults.page_height = page_layout.req_child("page-height").req_f32();
        layout.defaults.page_width = page_layout.req_child("page-width").req_f32();

        for pm in page_layout.children().filter(|n| n.has_tag("page-margins")) {
            let margins = PageMargins {
                left: pm.req_child("left-margin").req_f32(),
                right: pm.req_child("right-margin").req_f32(),
                top: pm.req_child("top-margin").req_f32(),
                bottom: pm.req_child("bottom-margin").req_f32(),
            };

            match pm.req_attr("type") {
                "both" => layout.page_margins_both = Some(margins),
                "odd" => layout.page_margins_odd = Some(margins),
                "even" => layout.page_margins_even = Some(margins),
                other => panic!("Invalid page-margins type '{}'", other),
            }
        }

        if let Some(appearance) = element.children().find(|n| n.has_tag("appearance")) {
            for lw in appearance.children().filter(|n| n.has_tag("line-width")) {
                let t = lw.req_attr("type");
                let v = lw.req_f32();

                match t {
                    "staff" => layout.staff_line_thickness = v,
                    "light barline" => layout.bar_line_light_thickness = v,
                    "heavy barline" => layout.bar_line_heavy_thickness = v,
                    _ => {}
                }
            }
        }

        if let Some(system_layout) = element.children().find(|n| n.has_tag("system-layout")) {
            if let Some(sm) = system_layout
                .children()
                .find(|n| n.has_tag("system-margins"))
            {
                layout.system_margin_left = sm.req_child("left-margin").req_f32();
                layout.system_margin_right = sm.req_child("right-margin").req_f32();
            }

            if let Some(n) = system_layout
                .children()
                .find(|n| n.has_tag("system-distance"))
            {
                layout.system_distance = n.req_f32();
            }

            if let Some(n) = system_layout
                .children()
                .find(|n| n.has_tag("top-system-distance"))
            {
                layout.top_system_distance = n.req_f32();
            }
        }

        if let Some(staff_layout) = element.children().find(|n| n.has_tag("staff-layout")) {
            let sd = staff_layout.req_child("staff-distance").req_f32();
            layout.staff_distance = sd;
        }
    }

    pub fn exit_defaults(&mut self, layout: &mut Layout, user_layout: &UserLayout) {
        layout.apply_user_layout(user_layout);
    }

    pub fn enter_part_list(&mut self, element: &Node, layout: &mut Layout) {
        layout.parts.clear();
        layout.sections.clear();

        let mut first_order_index: i32 = 0;
        let mut second_order_index: i32 = 0;

        let mut first_order_group_open = false;
        let mut second_order_group_open = false;

        for child in element.children().filter(|n| n.is_element()) {
            if child.has_tag_name("part-group") {
                let type_attr = child.req_attribute("type");

                match type_attr {
                    "start" if !first_order_group_open => {
                        let brace_type = brace_type(&child);

                        let group_first =
                            layout.sections.entry(first_order_index).or_default();
                        group_first.brace = brace_type;

                        first_order_group_open = true;

                        second_order_index = 0;
                        second_order_group_open = false;
                    }

                    "start" if !second_order_group_open => {
                        let group_first =
                            layout.sections.entry(first_order_index).or_default();

                        let name = child
                            .children()
                            .find(|n| n.has_tag_name("group-name"))
                            .and_then(|n| n.text())
                            .map(|s| s.to_string());

                        let brace = brace_type(&child);

                        let group_second =
                            group_first.groups.entry(second_order_index).or_default();

                        group_second.name = name;
                        group_second.brace = brace;

                        second_order_group_open = true;
                    }

                    "stop" if second_order_group_open => {
                        second_order_index += 1;
                        second_order_group_open = false;
                    }

                    "stop" if first_order_group_open => {
                        first_order_index += 1;
                        first_order_group_open = false;

                        second_order_index = 0;
                        second_order_group_open = false;
                    }

                    _ => {}
                }
            }

            if child.has_tag_name("score-part") {
                let id = child.req_attribute("id").to_string();

                let name = child
                    .children()
                    .find(|n| n.has_tag_name("part-name"))
                    .and_then(|n| n.text())
                    .unwrap_or("")
                    .to_string();

                let abbr = child
                    .children()
                    .find(|n| n.has_tag_name("part-abbreviation"))
                    .and_then(|n| n.text())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| name.clone());

                let part = Part {
                    name,
                    abbr,
                    section: first_order_index,
                    part_group: second_order_index,
                    brace: None,
                };

                layout.parts.insert(id, part);

                if !first_order_group_open {
                    first_order_index += 1;
                    second_order_index = 0;
                } else if !second_order_group_open {
                    second_order_index += 1;
                }
            }
        }
    }
}