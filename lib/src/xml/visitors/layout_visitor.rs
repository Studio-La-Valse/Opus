use crate::score::core::staff_idx::StaffIdx;
use crate::score::visual::brace::Brace;
use crate::score::visual::bracket::Bracket;
use crate::xml::visitor::Visitor;
use crate::xml::walker_ctx::WalkerCtx;

use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::system_measure::SystemMeasure;
use roxmltree::Node;
use std::collections::HashSet;

pub struct LayoutVisitor {
    pub encountered: HashSet<StaffIdx>,
}

impl LayoutVisitor {}

impl<'a> Visitor<WalkerCtx<'a>> for LayoutVisitor {
    fn enter(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_work(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_defaults(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn exit_defaults(&mut self, _ctx: &mut WalkerCtx) {}

    fn enter_part_list(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_part(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_measure(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {
        self.encountered.clear();
        self.encountered.insert(1.into());
    }

    fn enter_print(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_attributes(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_clef(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        self.encountered.insert(ctx.layout_ctx.staff.number);
    }

    fn enter_staff_details(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_backup(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        self.encountered.insert(ctx.layout_ctx.staff.number);
    }

    fn enter_forward(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        self.encountered.insert(ctx.layout_ctx.staff.number);
    }

    fn enter_note(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        self.encountered.insert(ctx.layout_ctx.staff.number);
    }

    fn exit_note(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit_measure(&mut self, ctx: &mut WalkerCtx) {
        let page_number = ctx.layout_ctx.page.page_number;
        let system_index = ctx.layout_ctx.system.index;
        let measure_number = ctx.layout_ctx.measure.number;

        let part_id = ctx.layout_ctx.part_id.clone();
        let assignment = ctx.layout.lookup(&part_id).unwrap();
        let section_number = assignment.section;
        let part_group_number = assignment.part_group;

        // get or create the page
        let page = ctx.visual_score.get_page_or_insert(page_number);

        // get or create the system on the page
        let system = page.get_system_or_insert(system_index);
        system.m_left = ctx.layout_ctx.system.margin_left.unwrap_or(system.m_left);
        system.m_right = ctx.layout_ctx.system.margin_right.unwrap_or(system.m_right);
        system.distance = ctx.layout_ctx.system.distance.unwrap_or(system.distance);
        system.top = ctx.layout_ctx.system.distance_top.unwrap_or(system.top);

        let system_measure = system
            .measures
            .entry(measure_number)
            .or_insert_with(|| SystemMeasure::new(measure_number));
        system_measure.init_width(ctx.layout_ctx.measure.width);

        // get or create the section in this system.
        let section = system.get_section_or_insert(section_number, || {
            let bracket_top = ctx.font.bracket_top();
            let bracket_bottom = ctx.font.bracket_bottom();
            Bracket::new(bracket_top, bracket_bottom)
        });

        // get or create the part group in this section.
        let part_group = section.part_group_or_insert(part_group_number, || {
            let smufl_brace = ctx.font.brace(None);
            Brace::new(smufl_brace)
        });

        // get or create the part in this part group.
        let part = part_group.part_or_insert(part_id.clone(), || Brace::new(ctx.font.brace(None)));
        part.measures
            .entry(measure_number)
            .or_insert_with(|| PartMeasure::new(part_id.clone(), measure_number));
        part.set_visibility(ctx.layout_ctx.part_hidden_specified);
        part.ensure_staves(&self.encountered);
        part.hide_staves(&ctx.layout_ctx.staff.explicitly_hidden);
        part.show_staves(&ctx.layout_ctx.staff.explicitly_shown);
        part.set_distances(&ctx.layout_ctx.staff.distances, &ctx.layout.staff_distance);

        // Now that all staves are ensure, consolidate the measure width,
        // so every measure from system to staff has the same calculated width.
        system.consolidate_measure_width(ctx.layout_ctx.measure.number);

        // Now that all measures are ensured, we can apply the scale,
        // because scale is passed to staff measures.
        let part = system.locate_part_mut(&part_id).unwrap();
        part.set_staff_scale(&ctx.layout_ctx.staff.staff_scaling);
    }

    fn exit_part(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit(&mut self, _ctx: &mut WalkerCtx) {}
}
