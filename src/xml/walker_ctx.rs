use crate::score::visual::score::Score;
use crate::{Layout, LayoutCtx, UserLayout};

pub struct WalkerCtx<'a> {
    pub user_layout: &'a mut UserLayout,
    pub layout: &'a mut Layout,
    pub layout_ctx: &'a mut LayoutCtx,
    pub visual_score: &'a mut Score,
}

impl<'a> WalkerCtx<'a> {
    pub fn new(
        user_layout: &'a mut UserLayout,
        layout: &'a mut Layout,
        layout_ctx: &'a mut LayoutCtx,
        visual_score: &'a mut Score,
    ) -> Self {
        Self {
            user_layout,
            layout,
            layout_ctx,
            visual_score,
        }
    }
}
