//! Places every note, rest, ledger line and opening/closing clef, key and
//! time signature in the tree, once the container pass ([`PageArranger`])
//! has placed every page, system, section, part-group, part, staff and
//! measure.
//!
//! Container geometry never reads content geometry -- a measure's width comes
//! from the walk and the `measure` pass, a staff's height from its line count,
//! a part's height from its staves -- so the container pass can run to
//! completion on its own, and content placement can be pulled out into its own
//! pass rather than being threaded through the container one.

use crate::score::visual::arrange_machine::ArrangeMachine;
use crate::score::visual::arranger::ScoreArranger;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::part::Part;
use crate::score::visual::score::Score;
use crate::score::visual::staff_measure::MeasureStartPaddings;
use crate::score::visual::system::System;

/// Rebuilds every content element's position from the containers the
/// [`PageArranger`](crate::score::visual::arranger::PageArranger) has already
/// placed.
///
/// The second of [`SCORE_ARRANGERS`](crate::score::visual::arranger::SCORE_ARRANGERS):
/// content placement needs every container's final position, and
/// [`BeamArranger`](crate::score::visual::arranger::BeamArranger),
/// [`TieArranger`](crate::score::visual::arranger::TieArranger) and
/// [`ClefChangeArranger`](crate::score::visual::arranger::ClefChangeArranger)
/// need the content it places.
pub struct ContentArranger;

impl ScoreArranger for ContentArranger {
    fn arrange(&self, score: &mut Score, params: LayoutParams<'_>) {
        let paddings = MeasureStartPaddings::resolve(params);

        for page in score.pages.values_mut() {
            for system in page.systems.values_mut() {
                walk_system(system, &paddings);
            }
        }
    }
}

// ---- internals ----

fn walk_system(system: &mut System, paddings: &MeasureStartPaddings) {
    for section in system.sections.values_mut() {
        for group in section.part_groups.values_mut() {
            for part in group.parts.values_mut() {
                walk_part(part);
            }
        }
    }

    // Last: the shared opening columns need every staff measure placed.
    ArrangeMachine.arrange_system_measure_starts(system, paddings);
}

fn walk_part(part: &mut Part) {
    let staff_ctx = part.create_staff_ctx();

    for staff in part.staves.values_mut() {
        if staff.hidden {
            continue;
        }

        for measure in staff.measures.values_mut() {
            ArrangeMachine.arrange_staff_measure_content(measure);
        }
    }

    // Not filtered on visibility, unlike every other content walk: a hidden
    // part's measures are still arranged today (`ArrangeMachine::arrange_part_clear_of` has
    // no such check) and nothing draws them, so this pass must keep leaving
    // them arranged rather than stale.
    for measure in part.measures.values_mut() {
        ArrangeMachine.arrange_part_measure_content(measure, &staff_ctx);
    }
}
