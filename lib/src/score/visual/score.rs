use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::page::Page;
use crate::visual::element::ScoreElement;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Score {
    pub pages: BTreeMap<u32, Page>,
}

impl Score {
    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for page in self.pages.values_mut() {
            page.rebeam(strategy);
        }
    }
}

impl ScoreElement for Score {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut result: Vec<&mut dyn ScoreElement> = Vec::new();

        for (_idx, staff) in self.pages.iter_mut() {
            result.push(staff);
        }

        result
    }
}

impl Layoutable for Score {
    fn measure(&mut self, available: &XY) {
        for (_idx, page) in self.pages.iter_mut() {
            page.measure(available);
        }
    }

    fn arrange(&mut self, origin: &XY) {
        let mut _origin: XY = *origin;

        for (_idx, page) in self.pages.iter_mut() {
            page.arrange(&_origin);
            _origin = _origin.mv(page.width, 0.);
            _origin = _origin.mv(200., 0.);
        }
    }
}

impl Content for Score {
    fn content(&self) -> Vec<&dyn Content> {
        self.pages.values().map(|s| s as &dyn Content).collect()
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}
