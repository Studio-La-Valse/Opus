use crate::musicxml::validation_issue::ValidationIssue;
use crate::score::walk_cursor::WalkCursor;

/// The context threaded through a validation walk. Deliberately holds only
/// what validation rules actually need: the position-tracking subset of
/// `WalkCursor` (reused so validation can never drift from what render
/// actually computes) and the issues collected along the way.
#[derive(Default)]
pub struct ValidationCtx {
    pub cursor: WalkCursor,
    pub issues: Vec<ValidationIssue>,
}
