use crate::musicxml::visitor::Visitor;
use crate::musicxml::walker_ctx::WalkerCtx;
use crate::score::core::staff_idx::StaffIdx;

use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::system_measure::SystemMeasure;
use roxmltree::Node;
use std::collections::HashSet;

pub struct LayoutVisitor {
    pub encountered: HashSet<StaffIdx>,
}

impl<'a> Visitor<WalkerCtx<'a>> for LayoutVisitor {
    fn enter_measure(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {
        self.encountered.clear();
        self.encountered.insert(1.into());
    }

    fn enter_clef(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        self.encountered.insert(ctx.cursor.staff.number);
    }

    fn enter_backup(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        self.encountered.insert(ctx.cursor.staff.number);
    }

    fn enter_forward(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        self.encountered.insert(ctx.cursor.staff.number);
    }

    fn enter_note(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        self.encountered.insert(ctx.cursor.staff.number);
    }

    fn exit_measure(&mut self, ctx: &mut WalkerCtx) {
        let page_number = ctx.cursor.page.page_number;
        let system_index = ctx.cursor.system.index;
        let measure_number = ctx.cursor.measure.number;

        let part_id = ctx.cursor.part_id.clone();
        let assignment = ctx.layout.lookup(&part_id).unwrap();
        let section_number = assignment.section;
        let part_group_number = assignment.part_group;

        // get or create the system on the page, applying this measure's system layout
        let system = ctx
            .visual_score
            .page_or_insert(page_number)
            .system_or_insert(system_index);
        system.m_left = ctx.cursor.system.margin_left.unwrap_or(system.m_left);
        system.m_right = ctx.cursor.system.margin_right.unwrap_or(system.m_right);
        system.distance = ctx.cursor.system.distance.unwrap_or(system.distance);
        system.top = ctx.cursor.system.distance_top.unwrap_or(system.top);

        let system_measure = system
            .measures
            .entry(measure_number)
            .or_insert_with(|| SystemMeasure::new(measure_number));
        system_measure.init_width(ctx.cursor.measure.width);

        // get or create section -> part group -> part below the system
        let part = ctx.visual_score.locate_or_create_part(
            ctx.font,
            page_number,
            system_index,
            section_number,
            part_group_number,
            &part_id,
        );
        part.measures
            .entry(measure_number)
            .or_insert_with(|| PartMeasure::new(part_id.clone(), measure_number));
        part.set_visibility(ctx.cursor.part_hidden_specified);
        part.ensure_staves(&self.encountered);
        part.hide_staves(&ctx.cursor.staff.explicitly_hidden);
        part.show_staves(&ctx.cursor.staff.explicitly_shown);
        part.set_distances(&ctx.cursor.staff.distances, &ctx.layout.staff_distance);

        // Now that all staves are ensure, consolidate the measure width,
        // so every measure from system to staff has the same calculated width.
        let system = ctx.visual_score.locate_system_mut(&system_index).unwrap();
        system.consolidate_measure_width(ctx.cursor.measure.number);

        // Now that all measures are ensured, we can apply the scale,
        // because scale is passed to staff measures.
        let part = system.locate_part_mut(&part_id).unwrap();
        part.set_staff_scale(&ctx.cursor.staff.staff_scaling);
    }
}
