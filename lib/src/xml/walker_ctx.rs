use crate::app_defaults::AppDefaults;
use crate::layout::Layout;
use crate::layout_ctx::LayoutCtx;
use crate::score::visual::score::Score;
use crate::smufl::smufl_font::SmuflFont;
use crate::user_layout::UserLayout;

pub struct WalkerCtx<'a> {
    pub user_layout: &'a UserLayout,
    pub layout: &'a mut Layout,
    pub app_defaults: &'a AppDefaults,
    pub layout_ctx: &'a mut LayoutCtx,
    pub visual_score: &'a mut Score,
    pub font: &'a SmuflFont,
}

impl<'a> WalkerCtx<'a> {
    pub fn new(
        user_layout: &'a UserLayout,
        layout: &'a mut Layout,
        app_defaults: &'a AppDefaults,
        layout_ctx: &'a mut LayoutCtx,
        visual_score: &'a mut Score,
        font: &'a SmuflFont,
    ) -> Self {
        Self {
            user_layout,
            layout,
            app_defaults,
            layout_ctx,
            visual_score,
            font,
        }
    }
}
