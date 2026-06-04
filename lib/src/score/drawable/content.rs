use crate::score::drawable::element::Element;

pub trait Content {
    fn content(&self) -> Vec<&dyn Content>;
    fn elements(&self) -> Vec<Element>;
}
