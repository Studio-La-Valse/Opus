use crate::musicxml::validation_issue::ValidationIssue;
use crate::score::layout_ctx::LayoutCtx;

/// The context threaded through a validation walk. Deliberately holds only
/// what validation rules actually need: the position-tracking subset of
/// `LayoutCtx` (reused so validation can never drift from what render
/// actually computes) and the issues collected along the way.
#[derive(Default)]
pub struct ValidationCtx {
    pub layout_ctx: LayoutCtx,
    pub issues: Vec<ValidationIssue>,
}
