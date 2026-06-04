use crate::core::xy::XY;
use crate::score::drawable::content::Content;
use crate::score::drawable::element::Element;
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::page::Page;
use std::collections::HashMap;

#[derive(Default)]
pub struct Score {
    pub pages: HashMap<u32, Page>,
}

impl Score {}

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
            _origin = _origin.mv(10., 0.);
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
