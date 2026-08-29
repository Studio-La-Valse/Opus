use crate::geometry::xy::XY;
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::score::Score;

pub trait LayoutEngine {
    fn arrange_pages(&self, score: &mut Score, origin: &XY);
}

pub struct HorizontalPageLayout {
    pub gutter_even: f32,
    pub gutter_uneven: f32,
}

impl LayoutEngine for HorizontalPageLayout {
    fn arrange_pages(&self, score: &mut Score, origin: &XY) {
        let mut cursor = *origin;
        for (i, page) in score.pages.values_mut().enumerate() {
            page.arrange(&cursor);
            let gutter = if i % 2 == 0 {
                self.gutter_even
            } else {
                self.gutter_uneven
            };
            cursor = cursor.mv(page.width + gutter, 0.);
        }
    }
}

pub struct VerticalPageLayout {
    pub gutter: f32,
}

impl LayoutEngine for VerticalPageLayout {
    fn arrange_pages(&self, score: &mut Score, origin: &XY) {
        let mut cursor = *origin;
        for page in score.pages.values_mut() {
            page.arrange(&cursor);
            cursor = cursor.mv(0., page.height + self.gutter);
        }
    }
}
