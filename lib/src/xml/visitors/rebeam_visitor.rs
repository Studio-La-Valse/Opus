use crate::visitor::Visitor;
use crate::xml::walker_ctx::WalkerCtx;
use roxmltree::Node;

pub struct RebeamVisitor {}

impl RebeamVisitor {}

impl Visitor for RebeamVisitor {
    fn enter(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_work(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_defaults(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn exit_defaults(&mut self, _ctx: &mut WalkerCtx) {}

    fn enter_part_list(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_part(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_measure(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_print(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_attributes(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_clef(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_staff_details(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_backup(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_forward(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_note(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn exit_note(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit_measure(&mut self, _ctx: &mut WalkerCtx) {
        let page_number = _ctx.layout_ctx.page.page_number;
        let system_index = _ctx.layout_ctx.system.index;
        let measure_number = _ctx.layout_ctx.measure.number;

        let part_id = _ctx.layout_ctx.part_id.to_string();
        let part = _ctx.layout.parts.get(&part_id).unwrap();
        let section_number = part.section;
        let part_group_number = part.part_group;

        let page = _ctx.visual_score.pages.get_mut(&page_number).unwrap();
        let system = page.systems.get_mut(&system_index).unwrap();
        let section = system.sections.get_mut(&section_number).unwrap();
        let part_group = section.part_groups.get_mut(&part_group_number).unwrap();
        let part = part_group.parts.get_mut(&part_id).unwrap();

        let part_measure = part.measures.get_mut(&measure_number).unwrap();
        if part_measure.requires_rebeam() {
            part_measure.rebeam();
        }
    }

    fn exit_part(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit(&mut self, _ctx: &mut WalkerCtx) {}
}
