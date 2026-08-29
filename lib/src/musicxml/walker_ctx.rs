use crate::score::app_defaults::AppDefaults;
use crate::score::score_defaults::ScoreDefaults;
use crate::score::user_layout::UserLayout;
use crate::score::visual::score::Score;
use crate::score::walk_cursor::WalkCursor;
use crate::smufl::smufl_font::SmuflFont;

pub struct WalkerCtx<'a> {
    pub user_layout: &'a UserLayout,
    pub layout: &'a mut ScoreDefaults,
    pub app_defaults: &'a AppDefaults,
    pub cursor: &'a mut WalkCursor,
    pub visual_score: &'a mut Score,
    pub font: &'a SmuflFont,
}

impl<'a> WalkerCtx<'a> {
    pub fn new(
        user_layout: &'a UserLayout,
        layout: &'a mut ScoreDefaults,
        app_defaults: &'a AppDefaults,
        cursor: &'a mut WalkCursor,
        visual_score: &'a mut Score,
        font: &'a SmuflFont,
    ) -> Self {
        Self {
            user_layout,
            layout,
            app_defaults,
            cursor,
            visual_score,
            font,
        }
    }
}
