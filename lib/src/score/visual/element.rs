use crate::layout::{Layout, UserLayout};

pub trait ScoreElement {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement>;

    fn apply_layout(&mut self, _layout: &Layout, _user_layout: &UserLayout) {
        for element in self.children() {
            element.apply_layout(_layout, _user_layout);
        }
    }
}
