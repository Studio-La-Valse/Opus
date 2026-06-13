use crate::layout::Layout;
use crate::layout_ctx::LayoutCtx;
use crate::score::visual::score::Score;

pub struct WalkerCtx<'a> {
    pub layout: &'a mut Layout,
    pub layout_ctx: &'a mut LayoutCtx,
    pub visual_score: &'a mut Score,
}

impl<'a> WalkerCtx<'a> {
    pub fn new(
        layout: &'a mut Layout,
        layout_ctx: &'a mut LayoutCtx,
        visual_score: &'a mut Score,
    ) -> Self {
        Self {
            layout,
            layout_ctx,
            visual_score,
        }
    }
}
