use crate::musicxml::validation_issue::ValidationIssue;
use crate::score::score_defaults::ScoreDefaults;
use crate::score::visual::score::Score;
use crate::score::walk_cursor::WalkCursor;

/// What the build walk reads and writes. Deliberately font-free: the walk
/// records which glyph each element draws by name, and the font is only
/// consulted once the score is arranged -- so a document can be walked before
/// its music font is chosen, and re-arranged in another font without a re-walk.
pub struct WalkerCtx<'a> {
    pub layout: &'a mut ScoreDefaults,
    pub cursor: &'a mut WalkCursor,
    pub visual_score: &'a mut Score,
    /// What the walk has to say for itself, the way `ValidationCtx::issues`
    /// carries what a validation walk has to say. A build walk reports no
    /// defects -- validation has already run and described any -- so in
    /// practice these are `Severity::Info`. Borrowed rather than owned because
    /// the two passes each get a fresh context and the messages outlive both.
    pub messages: &'a mut Vec<ValidationIssue>,
}

impl<'a> WalkerCtx<'a> {
    pub fn new(
        layout: &'a mut ScoreDefaults,
        cursor: &'a mut WalkCursor,
        visual_score: &'a mut Score,
        messages: &'a mut Vec<ValidationIssue>,
    ) -> Self {
        Self {
            layout,
            cursor,
            visual_score,
            messages,
        }
    }
}
