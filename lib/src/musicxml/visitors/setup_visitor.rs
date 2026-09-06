use crate::musicxml::utils::NodeUtils;
use crate::musicxml::utils::ReqParse;
use crate::musicxml::visitor::Visitor;
use crate::musicxml::walker_ctx::WalkerCtx;
use crate::score::part_list::builder::build_part_list;
use crate::score::score_defaults::PageMargins;
use roxmltree::Node;

pub struct SetupVisitor {}

impl<'a> Visitor<WalkerCtx<'a>> for SetupVisitor {
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

            // `type` is optional in MusicXML and defaults to "both".
            match pm.get_attribute("type").unwrap_or("both") {
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

    fn enter_part_list(&mut self, element: &Node, ctx: &mut WalkerCtx) {
        ctx.layout.part_list = build_part_list(element);
    }

    fn enter_part(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        // Ensure part is registered in part list
        let part_id = ctx.cursor.part_id.clone();
        ctx.layout.ensure_part(&part_id);
    }
}
