use crate::Visitor;
use crate::xml::walker_ctx::WalkerCtx;
use roxmltree::Node;
use crate::score::visual::note::Note;
use crate::score::visual::part_measure::PartMeasure;

pub struct ContentVisitor {
    pub part_measure: Option<PartMeasure>
}

impl ContentVisitor {}

impl Visitor for ContentVisitor {
    fn enter(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_work(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_defaults(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn exit_defaults(&mut self, _ctx: &mut WalkerCtx) {}

    fn enter_part_list(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_part(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_measure(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {
        self.part_measure = Some(PartMeasure::new());
    }

    fn enter_print(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_attributes(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_note(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {
        let part_measure = self.part_measure.as_mut().unwrap();

        let staff = _ctx.layout_ctx.staff.number;
        let note = Note { staff };
        part_measure.content.push(Box::new(note));
    }

    fn exit_measure(&mut self, _ctx: &mut WalkerCtx) {
        let part_measure = self.part_measure.take().unwrap();
        // set some properties on part_measure


    }

    fn exit_part(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit(&mut self, _ctx: &mut WalkerCtx) {}
}
