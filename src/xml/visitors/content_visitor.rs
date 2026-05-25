// use roxmltree::Node;
// use crate::{Layout, LayoutCtx, Visitor};
// use crate::score::visual::visual_score::VisualScore;
// 
// pub struct ContentVisitor<'a> {
//     visual_score: &'a mut VisualScore<'a>
// }
// 
// impl<'a> ContentVisitor<'a> {
//     pub fn new(visual_score: &'a mut VisualScore<'a>) -> Self {
//         Self {visual_score}
//     }
// }
// 
// impl<'a> Visitor for ContentVisitor<'a> {
// 
// 
//     fn enter(&mut self, _node: &Node) {
// 
//     }
// 
//     fn enter_work(&mut self, _node: &Node) {
// 
//     }
// 
//     fn enter_defaults(&mut self, _node: &Node) {
// 
//     }
// 
//     fn exit_defaults(&mut self) {
// 
//     }
// 
//     fn enter_part_list(&mut self, _node: &Node) {
// 
//     }
// 
//     fn enter_part(&mut self, _node: &Node) {
// 
//     }
// 
//     fn enter_measure(&mut self, _node: &Node) {
// 
//     }
// 
//     fn enter_print(&mut self, _node: &Node) {
// 
//     }
// 
//     fn enter_attributes(&mut self, _node: &Node) {
// 
//     }
// 
//     fn enter_note(&mut self, _node: &Node) {
// 
//     }
// 
//     fn exit_measure(&mut self) {
// 
//     }
// 
//     fn exit_part(&mut self) {
// 
//     }
// 
//     fn exit(&mut self) {
// 
//     }
// }