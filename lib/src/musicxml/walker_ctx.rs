use crate::musicxml::validation_issue::ValidationIssue;
use crate::score::layout_options::UserLayout;
use crate::score::score_defaults::ScoreDefaults;
use crate::score::visual::score::Score;
use crate::score::walk_cursor::WalkCursor;
use crate::smufl::smufl_font::SmuflFont;

pub struct WalkerCtx<'a> {
    pub user_layout: &'a UserLayout,
    pub layout: &'a mut ScoreDefaults,
    pub cursor: &'a mut WalkCursor,
    pub visual_score: &'a mut Score,
    pub font: &'a SmuflFont,
    /// What the walk has to say for itself, the way `ValidationCtx::issues`
    /// carries what a validation walk has to say. A build walk reports no
    /// defects -- validation has already run and described any -- so in
    /// practice these are `Severity::Info`. Borrowed rather than owned because
    /// the two passes each get a fresh context and the messages outlive both.
    pub messages: &'a mut Vec<ValidationIssue>,
}

impl<'a> WalkerCtx<'a> {
    pub fn new(
        user_layout: &'a UserLayout,
        layout: &'a mut ScoreDefaults,
        cursor: &'a mut WalkCursor,
        visual_score: &'a mut Score,
        font: &'a SmuflFont,
        messages: &'a mut Vec<ValidationIssue>,
    ) -> Self {
        Self {
            user_layout,
            layout,
            cursor,
            visual_score,
            font,
            messages,
        }
    }
}
