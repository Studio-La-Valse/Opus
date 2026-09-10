//! The parts of the arrange pass that walk a container's children in sequence,
//! where an off-by-one origin is invisible in the output until something starts
//! reading the coordinate it wrote.

#[cfg(test)]
mod tests {
    use lib::geometry::xy::XY;
    use lib::score::core::group_symbol::GroupLevel;
    use lib::score::visual::group_symbol::GroupSymbol;
    use lib::score::visual::layoutable::Layoutable;
    use lib::score::visual::part::Part;
    use lib::score::visual::part_group::PartGroup;
    use lib::score::visual::part_group_measure::PartGroupMeasure;
    use lib::score::visual::score::Score;
    use lib::score::visual::system::System;
    use lib::score::visual::system_measure::SystemMeasure;

    /// An unmeasured symbol: these tests are about where a container puts its
    /// children, and a symbol that never measures draws nothing.
    fn symbol(level: GroupLevel) -> GroupSymbol {
        GroupSymbol::new(level, None)
    }

    /// Measures tile left to right, so each one starts where the previous one
    /// ended. Passing the container's own origin to every measure instead would
    /// stack them all at x = 0.
    #[test]
    fn system_measures_are_laid_out_left_to_right() {
        let mut system = System::default();
        for (number, width) in [(1u32, 30.), (2, 50.), (3, 20.)] {
            let mut measure = SystemMeasure::new(number);
            measure.width = width;
            system.measures.insert(number, measure);
        }

        system.arrange(&XY { x: 100., y: 5. });

        let x_of = |n: u32| system.measures[&n].xy.x;
        assert_eq!(x_of(1), 100.);
        assert_eq!(x_of(2), 130.);
        assert_eq!(x_of(3), 180.);

        // The origin's y is shared by every measure; only x advances.
        assert!(system.measures.values().all(|m| m.xy.y == 5.));
    }

    #[test]
    fn part_group_measures_are_laid_out_left_to_right() {
        let mut group = PartGroup::new(symbol(GroupLevel::PartGroup));
        for (number, width) in [(1u32, 40.), (2, 60.)] {
            let measure = PartGroupMeasure {
                width,
                ..Default::default()
            };
            group.measures.insert(number, measure);
        }

        group.arrange(&XY { x: 10., y: 0. });

        assert_eq!(group.measures[&1].xy.x, 10.);
        assert_eq!(group.measures[&2].xy.x, 50.);
    }

    /// `create_staff_ctx` is what positions notes and rests, which hang off a
    /// `PartMeasure` and only name their staff by index. It has to walk the
    /// staves exactly the way `Part::arrange` places them, or the notes on a
    /// staff drift away from its own staff lines.
    #[test]
    fn staff_context_offsets_match_where_arrange_puts_the_staves() {
        let mut part = Part::new(symbol(GroupLevel::Part));
        for idx in [1u32, 2, 3] {
            let staff = part.staff_or_insert(&idx.into());
            staff.distance_final = 80.;
            staff.height = 40.;
        }
        // Hide the middle staff, the way `<staff-details print-object="no">` does.
        part.staff_or_insert(&2.into()).hidden = true;
        part.staff_or_insert(&2.into()).height = 0.;

        part.arrange(&XY::ZERO);
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
}
