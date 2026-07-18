use crate::drawable::drawable_element::DrawableElement;

pub trait DrawableContent {
    fn content(&self) -> Vec<&dyn DrawableContent>;
    fn elements(&self) -> Vec<DrawableElement>;
}
