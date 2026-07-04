use crate::app_defaults::AppDefaults;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::layout::Layout;
use crate::score::visual::layoutable::Layoutable;
use crate::user_layout::UserLayout;
use crate::visual::element::ScoreElement;

#[derive(Default)]
pub struct PartGroupMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,
}

impl PartGroupMeasure {}

impl ScoreElement for PartGroupMeasure {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        vec![]
    }

    fn _apply_layout(
        &mut self,
        _layout: &Layout,
        _user_layout: &UserLayout,
        _app_defaults: &AppDefaults,
    ) {
    }
}

impl Layoutable for PartGroupMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}

impl Content for PartGroupMeasure {
    fn content(&self) -> Vec<&dyn Content> {
        vec![]
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}
