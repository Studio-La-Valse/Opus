use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::score_defaults::ScoreDefaults;
use crate::score::user_layout::UserLayout;

/// The three layout-config sources every element resolves its own appearance
/// from, threaded through the [`Layoutable::measure`] pass: the document's
/// declared defaults, the caller's overrides, and the hard-coded fallbacks.
#[derive(Clone, Copy)]
pub struct LayoutParams<'a> {
    pub score_defaults: &'a ScoreDefaults,
    pub user_layout: &'a UserLayout,
    pub app_defaults: &'a AppDefaults,
}

pub trait Layoutable {
    /// Resolves this element's appearance from `params` and sizes it (and its
    /// children). This is the single downward pass that used to be a separate
    /// `apply_layout` walk followed by a size-only `measure` walk.
    fn measure(&mut self, available: &XY, params: LayoutParams<'_>);

    fn arrange(&mut self, origin: &XY);
}
