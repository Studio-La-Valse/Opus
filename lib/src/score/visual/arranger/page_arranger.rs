//! Runs the page-layout pass: placing every page -- and so every system
//! beneath it -- at its own, page-local origin.

use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::xy::XY;
use crate::score::core::group_symbol::GroupSymbol as Kind;
use crate::score::visual::arranger::ScoreArranger;
use crate::score::visual::group_name::GroupName;
use crate::score::visual::group_symbol::{Glyphs, GroupSymbol};
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::page::Page;
use crate::score::visual::part::Part;
use crate::score::visual::part_group::PartGroup;
use crate::score::visual::part_group_measure::PartGroupMeasure;
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::score::Score;
use crate::score::visual::section::Section;
use crate::score::visual::section_measure::SectionMeasure;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_measure::StaffMeasure;
use crate::score::visual::system::System;
use crate::score::visual::system_measure::SystemMeasure;

/// Places every page -- and so every system, measure and note beneath it -- at
/// its own origin, `XY::ZERO`. Coordinates are page-local: nothing compares a
/// coordinate on one page against a coordinate on another. Arranging pages
/// relative to each other is left to whatever consumes a rendered page --
/// the browser component stacks them with CSS, and the PDF and SVG sinks
/// already emit one physical page at a time.
///
/// The first of [`SCORE_ARRANGERS`](crate::score::visual::arranger::SCORE_ARRANGERS)
/// to place anything: nothing in the tree has a resolved coordinate until this has run, which is
/// exactly what beams, ties and mid-measure clef changes need from it.
pub struct PageArranger;

impl ScoreArranger for PageArranger {
    fn arrange(&self, score: &mut Score, _params: LayoutParams<'_>) {
        for page in score.pages.values_mut() {
            self.arrange_page(page, &XY::ZERO);
        }
    }
}

// ---- group symbol and name placement ----

impl PageArranger {
    /// `origin.x` is the left edge this symbol sits clear of, and `origin.y` the
    /// top line of the first staff it spans. Neither is a position for the
    /// symbol itself: it steps left by its own gap from there.
    ///
    /// What that edge is depends on the level. A section's is the system's own
    /// left edge, since a section symbol is the innermost of the three. A
    /// part-group's is whatever its section drew, and a part's is whatever its
    /// part-group drew, so the three stack outward without any of them knowing
    /// how wide the others are -- which they could not know, a brace's width
    /// following the span it covers.
    pub fn arrange_group_symbol(&self, symbol: &mut GroupSymbol, origin: &XY) {
        symbol.anchor = origin.mv(-symbol.gap, 0.);

        // One statement for both, so the box always describes the ink.
        let (shape, bounds) = if symbol.span <= 0. {
            symbol.empty_shape()
        } else {
            match (&symbol.glyphs, symbol.kind) {
                (Glyphs::Brace(glyph), Kind::Brace) => symbol.brace_shape(glyph),
                (Glyphs::Bracket(top, bottom), Kind::Bracket) => symbol.bracket_shape(top, bottom),
                (_, Kind::Line) => symbol.line_shape(),
                (_, Kind::Square) => symbol.square_shape(),
                // `None`, and any shape whose glyphs failed to resolve.
                _ => symbol.empty_shape(),
            }
        };

        symbol.shape = shape;
        symbol.bounds = bounds;
    }

    /// Places the box in one statement, so it always describes what is reserved.
    ///
    /// `top` is the top line of the first staff the name covers, `right` the
    /// left edge of the level's symbol (or where that symbol would have been,
    /// when nothing is drawn), and `margin_left` the page's left margin. The
    /// box's right edge is always `right` less the padding -- that is where a
    /// right-aligned run ends. Its left edge is the margin when there is room,
    /// and the right edge itself when there is not: a name with no room to its
    /// left still ends in the right place and simply overflows past the margin,
    /// since nothing in the horizontal layout moves to make space for it.
    pub fn arrange_group_name_between(
        &self,
        name: &mut GroupName,
        top: XY,
        right: f32,
        margin_left: f32,
    ) {
        let box_right = right - name.padding;
        let box_left = margin_left.min(box_right);
        name.bounds = BoundingBox {
            xy: XY {
                x: box_left,
                y: top.y,
            },
            size: XY {
                x: box_right - box_left,
                y: name.span,
            },
        };
    }
}

// ---- internals ----

impl PageArranger {
    /// Places this page's systems below its margins.
    fn arrange_page(&self, page: &mut Page, origin: &XY) {
        let m_left = page.margins.left;
        let m_top = page.margins.top;

        // top left of available space after margins
        let mut origin = origin.mv(m_left, m_top);

        let mut first = true;
        for system in page.systems.values_mut() {
            let s_m_left = system.m_left;
            let s_left = origin.x + s_m_left;

            // space on top of system is either its margin to previous if any,
            // else the distance to top of margins
            let mut s_m_top = system.distance;
            if first {
                s_m_top = system.top;
                first = false
            }

            let s_top = origin.y + s_m_top;

            let s_origin = XY {
                x: s_left,
                y: s_top,
            };
            self.arrange_system(system, &s_origin);

            origin = origin.mv(0., system.height + s_m_top);
        }
    }

    fn arrange_system(&self, system: &mut System, origin: &XY) {
        system.xy = *origin;

        let mut _origin = system.xy;
        for measure in system.measures.values_mut() {
            self.arrange_system_measure(measure, &_origin);
            _origin = _origin.mv(measure.width, 0.);
        }

        // The page's left margin, in world coordinates: `arrange_page` places a
        // system at `page_margin + system.m_left`, so undoing the system's own
        // margin lands back on it. This is where a name's box reaches left to.
        let margin_left = system.xy.x - system.m_left;

        let mut _origin = system.xy;
        for section in system.sections.values_mut() {
            self.arrange_section_within(section, &_origin, margin_left);
            _origin = _origin.mv(0., section.height);
        }
    }

    fn arrange_system_measure(&self, measure: &mut SystemMeasure, _origin: &XY) {
        measure.xy = *_origin;
    }

    /// Places this section, additionally carrying the page's left margin
    /// (`margin_left`) down the chain so a part / part-group name knows how far
    /// left its box reaches. A section has no name of its own -- it is the
    /// (usually unnamed) bracket around a run of part-groups.
    fn arrange_section_within(&self, section: &mut Section, origin: &XY, margin_left: f32) {
        section.xy = *origin;

        let first_visible_staff_distance = section.first_visible_staff_distance();
        let mut measure_origin = section.xy.mv(0., first_visible_staff_distance);

        for measure in section.measures.values_mut() {
            self.arrange_section_measure(measure, &measure_origin);
            measure_origin = measure_origin.mv(measure.width, 0.);
        }

        // Arranged before the part-groups, because where they put their own
        // symbols depends on how far left this one reached. A section's symbol
        // is the innermost of the three, so it is the only one measured from
        // the system itself.
        self.arrange_group_symbol(
            &mut section.symbol,
            &section.xy.mv(0., first_visible_staff_distance),
        );
        let clear_of = self.section_symbol_left_edge(section);

        let mut part_group_origin = section.xy;
        for part_group in section.part_groups.values_mut() {
            self.arrange_part_group_clear_of(part_group, &part_group_origin, clear_of, margin_left);
            part_group_origin = part_group_origin.mv(0., part_group.height);
        }
    }

    fn arrange_section_measure(&self, measure: &mut SectionMeasure, origin: &XY) {
        measure.xy = *origin;
    }

    /// Places this group, with `clear_of` the left edge of whatever the
    /// enclosing section drew and `margin_left` the page's left margin. This
    /// group's symbol sits its own gap further out than `clear_of`, its name
    /// ends a padding left of the symbol and reaches back to `margin_left`, and
    /// its parts keep clear of the symbol in turn -- so the three levels stack
    /// outward from the system without any of them knowing how wide the others
    /// are.
    fn arrange_part_group_clear_of(
        &self,
        group: &mut PartGroup,
        origin: &XY,
        clear_of: f32,
        margin_left: f32,
    ) {
        group.xy = *origin;

        let first_visible_staff_distance = group.first_visible_staff_distance();

        let mut _origin = group.xy;
        for measure in group.measures.values_mut() {
            self.arrange_part_group_measure(measure, &_origin);
            _origin = _origin.mv(measure.width, 0.);
        }

        // Before the parts, whose own symbols keep clear of this one.
        let symbol_top = XY {
            x: clear_of,
            y: group.xy.y + first_visible_staff_distance,
        };
        self.arrange_group_symbol(&mut group.symbol, &symbol_top);
        let clear_of = self.part_group_symbol_left_edge(group, clear_of);

        self.arrange_group_name_between(&mut group.name, symbol_top, clear_of, margin_left);

        let mut _origin = group.xy;
        for part in group.parts.values_mut() {
            self.arrange_part_clear_of(part, &_origin, clear_of, margin_left);
            _origin = _origin.mv(0., part.height);
        }
    }

    fn arrange_part_group_measure(&self, measure: &mut PartGroupMeasure, origin: &XY) {
        measure.xy = *origin;
    }

    /// Places this part, with `clear_of` the left edge of whatever its
    /// part-group drew and `margin_left` the page's left margin. A part's
    /// symbol is the outermost of the three, so nothing keeps clear of it in
    /// turn; its name ends a padding left of it and reaches back to the margin.
    fn arrange_part_clear_of(&self, part: &mut Part, origin: &XY, clear_of: f32, margin_left: f32) {
        part.xy = *origin;

        let mut _origin = part.xy;
        for staff in part.staves.values_mut() {
            if staff.hidden {
                continue;
            }

            _origin = _origin.mv(0., staff.distance_final);

            self.arrange_staff(staff, &_origin);
            _origin = _origin.mv(0., staff.height());
        }

        let mut _origin = part.xy;
        for measure in part.measures.values_mut() {
            self.arrange_part_measure(measure, &_origin);
            _origin = _origin.mv(measure.width, 0.);
        }

        let first_visible_staff_distance = part.first_visible_staff_distance();
        let symbol_top = XY {
            x: clear_of,
            y: part.xy.y + first_visible_staff_distance,
        };
        self.arrange_group_symbol(&mut part.symbol, &symbol_top);

        let name_right = self.part_symbol_left_edge(part, clear_of);
        self.arrange_group_name_between(&mut part.name, symbol_top, name_right, margin_left);
    }

    fn arrange_part_measure(&self, measure: &mut PartMeasure, origin: &XY) {
        measure.origin = *origin;
    }

    fn arrange_staff(&self, staff: &mut Staff, origin: &XY) {
        staff.xy = *origin;

        let mut _origin = staff.xy;
        for measure in staff.measures.values_mut() {
            self.arrange_staff_measure(measure, &_origin);

            _origin = _origin.mv(measure.width, 0.)
        }
    }

    /// Places this measure's own position. Everything it draws is placed
    /// afterwards by
    /// [`arrange_staff_measure_content`](crate::score::visual::arranger::ContentArranger::arrange_staff_measure_content),
    /// which this pass does not call: content placement is
    /// [`ContentArranger`](crate::score::visual::arranger::ContentArranger)'s
    /// job, run once every container in the tree -- this measure's opening
    /// columns included -- has its final position.
    fn arrange_staff_measure(&self, measure: &mut StaffMeasure, origin: &XY) {
        measure.xy = *origin;
    }

    /// How far left this section's own ink reaches, which is what the symbols
    /// inside it keep clear of. The system's left edge when nothing is drawn --
    /// an undrawn symbol must not push its part-groups outward by a gap that
    /// nothing occupies.
    fn section_symbol_left_edge(&self, section: &Section) -> f32 {
        if section.shows_symbol() {
            section.symbol.bounds().x_min()
        } else {
            section.xy.x
        }
    }

    /// How far left this group's own ink reaches, which is what its name aligns
    /// against and what its parts keep clear of: the symbol's left edge when it
    /// draws, and the incoming `clear_of` when it does not -- an undrawn symbol
    /// must not push what is outside it away by a gap nothing occupies. The
    /// counterpart of [`section_symbol_left_edge`](Self::section_symbol_left_edge).
    fn part_group_symbol_left_edge(&self, group: &PartGroup, clear_of: f32) -> f32 {
        if group.shows_symbol() {
            group.symbol.bounds().x_min()
        } else {
            clear_of
        }
    }

    /// How far left this part's own ink reaches, which is what its name aligns
    /// against: the symbol's left edge when it draws, and the incoming
    /// `clear_of` when it does not. The counterpart of
    /// [`section_symbol_left_edge`](Self::section_symbol_left_edge).
    fn part_symbol_left_edge(&self, part: &Part, clear_of: f32) -> f32 {
        if part.shows_symbol() {
            part.symbol.bounds().x_min()
        } else {
            clear_of
        }
    }
}
