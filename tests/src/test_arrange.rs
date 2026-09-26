//! The parts of the arrange pass that walk a container's children in sequence,
//! where an off-by-one origin is invisible in the output until something starts
//! reading the coordinate it wrote.

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use std::sync::OnceLock;

    use lib::score::core::group_symbol::GroupLevel;
    use lib::score::layout_options::UserLayout;
    use lib::score::score_defaults::{PageMargins, ScoreDefaults};
    use lib::score::visual::arranger::{PageArranger, ScoreArranger};
    use lib::score::visual::group_name::GroupName;
    use lib::score::visual::group_symbol::GroupSymbol;
    use lib::score::visual::layoutable::LayoutParams;
    use lib::score::visual::part_group_measure::PartGroupMeasure;
    use lib::score::visual::score::Score;
    use lib::score::visual::system_measure::SystemMeasure;
    use lib::smufl::smufl_font::SmuflFont;

    const BRAVURA_META: &str = "assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json";
    const GLYPH_NAMES: &str = "assets/smufl/metadata/glyphnames.json";

    fn fixture(relative: &str) -> String {
        read_to_string(format!("{}/../{relative}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative}: {e}"))
    }

    fn font() -> &'static SmuflFont {
        static FONT: OnceLock<SmuflFont> = OnceLock::new();
        FONT.get_or_init(|| SmuflFont::load(&fixture(BRAVURA_META), &fixture(GLYPH_NAMES)))
    }

    /// An unmeasured symbol: these tests are about where a container puts its
    /// children, and a symbol that never measures draws nothing.
    fn symbol(level: GroupLevel) -> GroupSymbol {
        GroupSymbol::new(level, None)
    }

    /// An unnamed name: these tests are about child placement, and a name with
    /// nothing to draw draws nothing.
    fn name() -> GroupName {
        GroupName::new(String::new(), String::new(), false)
    }

    /// A score of one page with no margins, so that a system's own `m_left` and
    /// `top` are exactly the origin it is arranged at.
    fn score_with_a_bare_page() -> Score {
        let mut score = Score::default();
        score.page_or_insert(1).margins = PageMargins {
            left: 0.,
            right: 0.,
            top: 0.,
            bottom: 0.,
        };
        score
    }

    fn arrange_pages(score: &mut Score) {
        let score_defaults = ScoreDefaults::default();
        let user_layout = UserLayout::default();
        let params = LayoutParams {
            score_defaults: &score_defaults,
            user_layout: &user_layout,
            font: font(),
        };

        PageArranger.arrange(score, params);
    }

    /// Measures tile left to right, so each one starts where the previous one
    /// ended. Passing the container's own origin to every measure instead would
    /// stack them all at x = 0.
    #[test]
    fn system_measures_are_laid_out_left_to_right() {
        let mut score = score_with_a_bare_page();
        let system = score.page_or_insert(1).system_or_insert(1);
        system.m_left = 100.;
        system.top = 5.;
        for (number, width) in [(1u32, 30.), (2, 50.), (3, 20.)] {
            let mut measure = SystemMeasure::new(number);
            measure.width = width;
            system.measures.insert(number, measure);
        }

        arrange_pages(&mut score);

        let system = &score.pages[&1].systems[&1];
        let x_of = |n: u32| system.measures[&n].xy.x;
        assert_eq!(x_of(1), 100.);
        assert_eq!(x_of(2), 130.);
        assert_eq!(x_of(3), 180.);

        // The origin's y is shared by every measure; only x advances.
        assert!(system.measures.values().all(|m| m.xy.y == 5.));
    }

    #[test]
    fn part_group_measures_are_laid_out_left_to_right() {
        let mut score = score_with_a_bare_page();
        let system = score.page_or_insert(1).system_or_insert(1);
        system.m_left = 10.;
        let section = system.section_or_insert(1, symbol(GroupLevel::Section));
        let group = section.part_group_or_insert(1, symbol(GroupLevel::PartGroup), name());
        for (number, width) in [(1u32, 40.), (2, 60.)] {
            let measure = PartGroupMeasure {
                width,
                ..Default::default()
            };
            group.measures.insert(number, measure);
        }

        arrange_pages(&mut score);

        let group = &score.pages[&1].systems[&1].sections[&1].part_groups[&1];
        assert_eq!(group.measures[&1].xy.x, 10.);
        assert_eq!(group.measures[&2].xy.x, 50.);
    }

    /// `create_staff_ctx` is what positions notes and rests, which hang off a
    /// `PartMeasure` and only name their staff by index. It has to walk the
    /// staves exactly the way `PageArranger` places them, or the notes on a
    /// staff drift away from its own staff lines.
    #[test]
    fn staff_context_offsets_match_where_arrange_puts_the_staves() {
        let mut score = score_with_a_bare_page();
        let system = score.page_or_insert(1).system_or_insert(1);
        let section = system.section_or_insert(1, symbol(GroupLevel::Section));
        let group = section.part_group_or_insert(1, symbol(GroupLevel::PartGroup), name());
        let part = group.part_or_insert("P1".to_string(), symbol(GroupLevel::Part), name());
        for idx in [1u32, 2, 3] {
            let staff = part.staff_or_insert(&idx.into());
            staff.distance_final = 80.;
            staff.height = 40.;
        }
        // Hide the middle staff, the way `<staff-details print-object="no">` does.
        part.staff_or_insert(&2.into()).hidden = true;
        part.staff_or_insert(&2.into()).height = 0.;

        arrange_pages(&mut score);

        let part = &score.pages[&1].systems[&1].sections[&1].part_groups[&1].parts["P1"];
        let ctx = part.create_staff_ctx();

        for (idx, staff) in part.staves.iter() {
            if staff.hidden {
                continue;
            }
            assert_eq!(
                ctx[idx].distance_from_top, staff.xy.y,
                "staff {idx:?} context disagrees with its arranged position"
            );
        }

        // Concretely: staff 3 follows staff 1 directly, the hidden staff between
        // them contributing neither its distance nor its height.
        assert_eq!(ctx[&1.into()].distance_from_top, 80.);
        assert_eq!(ctx[&3.into()].distance_from_top, 200.);
    }

    /// `Page::resolve_layout` picks odd- or even-page margins from `Page::number`,
    /// so the page has to actually know which one it is.
    #[test]
    fn a_created_page_knows_its_own_number() {
        let mut score = Score::default();

        for number in [1u32, 2, 7] {
            assert_eq!(score.page_or_insert(number).number, number);
        }

        // Re-fetching an existing page leaves its number alone.
        assert_eq!(score.page_or_insert(2).number, 2);
        assert_eq!(score.pages.len(), 3);
    }

    /// Pages no longer sit relative to each other -- the browser arranges
    /// them, not the engine -- so every page in a multi-page score must arrange
    /// as if it were the only page: its first system lands at that page's own
    /// margins, not at an offset accumulated from earlier pages' widths,
    /// heights or gutters. `Page` has no field left to assert a page's own
    /// position against (that was the point of removing it: every page is
    /// engraved at `XY::ZERO`, definitionally, by `PageArranger`'s `for page in
    /// score.pages.values_mut() { self.arrange_page(page, &XY::ZERO) }` loop) -- so this
    /// test's only way to observe cross-page drift is indirectly, through
    /// where a page's own content actually lands. Distinct, nonzero
    /// margins/width/height per page is what makes that observable: were
    /// `PageArranger` to regress into arranging pages one after another again
    /// (summing widths and gutters as it used to), a later page's system would
    /// land past its own margin rather than exactly on it.
    #[test]
    fn page_arranger_lands_each_pages_first_system_at_that_pages_own_margins() {
        let mut score = Score::default();

        for (number, left, top, width, height) in [
            (1u32, 30., 20., 300., 400.),
            (2, 50., 40., 500., 600.),
            (3, 10., 5., 200., 250.),
        ] {
            let page = score.page_or_insert(number);
            page.margins = PageMargins {
                left,
                right: 0.,
                top,
                bottom: 0.,
            };
            page.width = width;
            page.height = height;

            let system = page.system_or_insert(1);
            system.top = 15.;
            system.m_left = 5.;
        }

        arrange_pages(&mut score);

        for page in score.pages.values() {
            let system = &page.systems[&1];
            assert_eq!(
                system.xy.x,
                page.margins.left + system.m_left,
                "page {}'s first system must land at that page's own left margin, \
                 not one drifted by an earlier page's width",
                page.number,
            );
            assert_eq!(
                system.xy.y,
                page.margins.top + system.top,
                "page {}'s first system must land at that page's own top margin, \
                 not one drifted by an earlier page's height",
                page.number,
            );
        }
    }
}
