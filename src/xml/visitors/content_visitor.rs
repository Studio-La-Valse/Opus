use crate::Visitor;
use crate::score::visual::note::Note;
use crate::score::visual::page::Page;
use crate::score::visual::part_measure::PartMeasure;
use crate::xml::walker_ctx::WalkerCtx;
use roxmltree::Node;
use std::collections::HashMap;

pub struct ContentVisitor {
    pub part_measure: Option<PartMeasure>,
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

        let page_number = _ctx.layout_ctx.page.page_number;
        let system_index = _ctx.layout_ctx.system.index;

        let part_id = _ctx.layout_ctx.part_id.clone();
        let part = _ctx.layout.parts.get(&part_id).unwrap();
        let section_number = part.section;
        let part_group_number = part.part_group;
        let section = _ctx.layout.sections.entry(section_number).or_default();
        let _part_group = section.groups.entry(part_group_number).or_default();

        // get or create the page
        let page = _ctx
            .visual_score
            .pages
            .entry(page_number)
            .or_insert_with(|| Page {
                number: page_number,
                color: _ctx.layout.page_color,
                foreground: _ctx.layout.foreground_color,
                margins: _ctx.layout.get_margins(page_number),
                systems: HashMap::new(),
            });

        // get or create the system on the page
        let system = page.systems.entry(system_index).or_default();
        system.m_left = _ctx.layout_ctx.system.margin_left.unwrap_or(system.m_left);
        system.m_right = _ctx
            .layout_ctx
            .system
            .margin_right
            .unwrap_or(system.m_right);
        system.distance = _ctx.layout_ctx.system.distance.unwrap_or(system.distance);
        system.top = _ctx.layout_ctx.system.distance_top.unwrap_or(system.top);

        // get or create the section in this system.
        let section = system.sections.entry(section_number).or_default();

        // get or create the part group in this section.
        let part_group = section.part_groups.entry(part_group_number).or_default();

        // get or create the part in this part group.
        let part = part_group.parts.entry(part_id).or_default();
        part.set_visibility(_ctx.layout_ctx.part_hidden_specified);
        part.ensure_staves(part_measure.content.iter().map(|c| c.staff()).collect());
        part.hide_staves(&_ctx.layout_ctx.staff.explicitly_hidden);
        part.show_staves(&_ctx.layout_ctx.staff.explicitly_shown);
        part.set_distances(&_ctx.layout_ctx.staff.distances);
    }

    fn exit_part(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit(&mut self, _ctx: &mut WalkerCtx) {}
}
