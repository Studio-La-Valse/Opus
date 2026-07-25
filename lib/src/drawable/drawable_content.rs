use crate::drawable::drawable_element::DrawableElement;

pub trait Drawable {
    fn content(&self) -> Vec<&dyn Drawable>;
    fn elements(&self) -> Vec<DrawableElement>;
}
