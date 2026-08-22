use crate::score::layout::{PageMargins, Part, PartGroupLevel, PartListTracker};
use crate::xml::utils::NodeUtils;
use crate::xml::utils::ReqParse;
use crate::xml::visitor::Visitor;
use crate::xml::walker_ctx::WalkerCtx;
use roxmltree::Node;

pub struct SetupVisitor {}

impl SetupVisitor {}

impl<'a> Visitor<WalkerCtx<'a>> for SetupVisitor {
    fn enter(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_work(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        ctx.layout.work_title = element.req_child("work-title").req_parse();
    }

    fn enter_defaults(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        let scaling = element.req_child("scaling");
        ctx.layout.defaults.scaling_millimeters = scaling.req_child("millimeters").req_parse();
        ctx.layout.defaults.scaling_tenths = scaling.req_child("tenths").req_parse();

        let page_layout = element.req_child("page-layout");
        ctx.layout.defaults.page_height = page_layout.req_child("page-height").req_parse();
        ctx.layout.defaults.page_width = page_layout.req_child("page-width").req_parse();

        for pm in page_layout.children().filter(|n| n.has_tag("page-margins")) {
            let margins = PageMargins {
                left: pm.req_child("left-margin").req_parse(),
                right: pm.req_child("right-margin").req_parse(),
                top: pm.req_child("top-margin").req_parse(),
                bottom: pm.req_child("bottom-margin").req_parse(),
            };

            match pm.req_attribute("type") {
                "both" => ctx.layout.page_margins_both = Some(margins),
                "odd" => ctx.layout.page_margins_odd = Some(margins),
                "even" => ctx.layout.page_margins_even = Some(margins),
                other => panic!("Invalid page-margins type '{}'", other),
            }
        }

        if let Some(appearance) = element.children().find(|n| n.has_tag("appearance")) {
            for lw in appearance.children().filter(|n| n.has_tag("line-width")) {
                let t = lw.req_attribute("type");
                let v: f32 = lw.req_parse();

                match t {
                    "staff" => ctx.layout.appearance.staff = Some(v),
                    "light barline" => ctx.layout.appearance.light_barline = Some(v),
                    "heavy barline" => ctx.layout.appearance.heavy_barline = Some(v),
                    "beam" => ctx.layout.appearance.beam_thickness = Some(v),
                    "stem" => ctx.layout.appearance.stem_thickness = Some(v),
                    _ => {}
                }
            }
        }

        if let Some(system_layout) = element.children().find(|n| n.has_tag("system-layout")) {
            if let Some(sm) = system_layout
                .children()
                .find(|n| n.has_tag("system-margins"))
            {
                ctx.layout.system_margin_left = sm.req_child("left-margin").req_parse();
                ctx.layout.system_margin_right = sm.req_child("right-margin").req_parse();
            }

            if let Some(n) = system_layout
                .children()
                .find(|n| n.has_tag("system-distance"))
            {
                ctx.layout.system_distance = n.req_parse();
            }

            if let Some(n) = system_layout
                .children()
                .find(|n| n.has_tag("top-system-distance"))
            {
                ctx.layout.top_system_distance = n.req_parse();
            }
        }

        if let Some(staff_layout) = element.children().find(|n| n.has_tag("staff-layout")) {
            let sd = staff_layout.req_child("staff-distance").req_parse();
            ctx.layout.staff_distance = sd;
        }
    }

    fn exit_defaults(&mut self, _ctx: &mut WalkerCtx) {}

    fn enter_part_list(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        ctx.layout.parts.clear();
        ctx.layout.sections.clear();

        let mut tracker = PartListTracker::default();

        for child in element.children().filter(|n| n.is_element()) {
            if child.has_tag_name("part-group") {
                match child.req_attribute("type") {
                    "start" => match tracker.open_part_group() {
                        Some(PartGroupLevel::Section { index }) => {
                            let name = child
                                .children()
                                .find(|n| n.has_tag_name("group-name"))
                                .and_then(|n| n.text())
                                .map(|s| s.to_string());

                            let brace_type = brace_type(&child);

                            let group_first = ctx.layout.sections.entry(index).or_default();
                            group_first.name = name;
                            group_first.brace = brace_type;
                        }
                        Some(PartGroupLevel::Group { section, index }) => {
                            let group_first = ctx.layout.sections.entry(section).or_default();

                            let name = child
                                .children()
                                .find(|n| n.has_tag_name("group-name"))
                                .and_then(|n| n.text())
                                .map(|s| s.to_string());

                            let brace = brace_type(&child);

                            let group_second = group_first.groups.entry(index).or_default();

                            group_second.name = name;
                            group_second.brace = brace;
                        }
                        None => {}
                    },
                    "stop" => tracker.close_part_group(),
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

                let (section, part_group) = tracker.register_score_part();

                let part = Part {
                    name,
                    abbr,
                    section,
                    part_group,
                    brace: None,
                };

                ctx.layout.parts.insert(id, part);
            }
        }
    }

    fn enter_part(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        // Ensure part is registered in part list
        let part_id = ctx.layout_ctx.part_id.clone();
        let part = ctx.layout.parts.get(&part_id).unwrap();

        // Ensure section is registered
        let section_number = part.section;
        let section = ctx.layout.sections.entry(section_number).or_default();

        // Ensure part group is registered
        let part_group_number = part.part_group;
        section.groups.entry(part_group_number).or_default();
    }

    fn enter_measure(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_print(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_attributes(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_clef(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_staff_details(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_backup(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_forward(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_note(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn exit_note(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit_measure(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit_part(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit(&mut self, _ctx: &mut WalkerCtx) {}
}

fn brace_type(node: &Node) -> Option<String> {
    node.children()
        .find(|n| n.has_tag_name("group-symbol"))
        .and_then(|n| n.text())
        .map(|s| s.to_string())
}
