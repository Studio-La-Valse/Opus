use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::layoutable::LayoutParams;

/// The instrument or part-group name drawn to the left of a group symbol: a
/// part's `<part-name>`, or a part-group's `<group-name>`.
///
/// One element for both levels, the way [`GroupSymbol`] is one element for all
/// three -- a part and a part-group name the same way and are drawn the same
/// way, so the level is the caller's business.
///
/// The box this reports is a *reserved* box, not a measurement of the ink: this
/// crate has no font metrics for a text face, so the box is the rectangle the
/// name is laid out in and it is the renderer's target's business whether the
/// text fills it -- see [`Text`](crate::drawable::elements::text::Text). Its
/// right edge is the level's symbol left edge less a configured padding; its
/// left edge is the page's left margin; a name longer than the box overflows to
/// the left and nothing in the horizontal layout moves for it.
///
/// [`bounds`](Self::bounds) is written by a single statement in
/// [`arrange_between`](Self::arrange_between) and is readable but not writable,
/// the invariant-by-construction shape [`GroupSymbol`] already uses.
///
/// [`GroupSymbol`]: crate::score::visual::group_symbol::GroupSymbol
#[derive(Clone)]
pub struct GroupName {
    name: String,
    abbr: String,

    // Resolved on every layout pass, like a `GroupSymbol`'s appearance.
    color: Color,
    font_size: f32,
    padding: f32,
    abbreviated: bool,

    span: f32,
    bounds: BoundingBox,
}

impl GroupName {
    /// A name that will draw `name` in full, or `abbr` on systems past the
    /// first. Either may be empty: a part with no `<part-name>` draws nothing.
    /// `abbreviate` is fixed at construction -- which system this name belongs
    /// to never changes -- and comes from the walk cursor's system index, the
    /// way [`System::index`](crate::score::visual::system::System::index)
    /// itself does.
    pub fn new(name: String, abbr: String, abbreviate: bool) -> Self {
        Self {
            name,
            abbr,
            color: Color::BLACK,
            font_size: 0.,
            padding: 0.,
            abbreviated: abbreviate,
            span: 0.,
            bounds: BoundingBox::ZERO,
        }
    }

    /// The string this name draws on the current pass: the abbreviation on a
    /// system that asked for one and has a non-empty abbreviation, the full
    /// name otherwise.
    pub fn text(&self) -> &str {
        if self.abbreviated && !self.abbr.is_empty() {
            &self.abbr
        } else {
            &self.name
        }
    }

    /// The colour the name is drawn in -- the score foreground, as a
    /// [`GroupSymbol`](crate::score::visual::group_symbol::GroupSymbol)'s ink is.
    pub fn color(&self) -> Color {
        self.color
    }

    /// The resolved font size, in tenths.
    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    /// The box reserved for the name, in world coordinates. Right-aligned and
    /// vertically centred text is drawn from this box's right-middle point.
    pub fn bounds(&self) -> BoundingBox {
        self.bounds
    }

    /// The point a right-aligned, vertically-centred run is drawn from: the
    /// box's right edge, at its vertical centre.
    pub fn anchor(&self) -> XY {
        XY {
            x: self.bounds.x_max(),
            y: self.bounds.y_min() + self.bounds.height() / 2.,
        }
    }

    /// Whether this name has anything to draw: a non-empty string. Whether
    /// there are staves for it to sit beside is the caller's question --
    /// [`Part::shows_name`](crate::score::visual::part::Part) and
    /// [`PartGroup::shows_name`](crate::score::visual::part_group::PartGroup)
    /// both gate on the visible-staff count -- and it is deliberately not asked
    /// here: a one-line percussion staff is zero tenths tall, so [`span`] is
    /// legitimately zero for it, and the name still belongs centred on that
    /// line.
    ///
    /// [`span`]: Self::span
    pub fn is_drawn(&self) -> bool {
        !self.text().is_empty()
    }

    /// Resolves the name's appearance from `params`: colour, size and padding.
    /// Whether this name draws its abbreviation is settled at construction, not
    /// here -- see [`new`](Self::new). Mirrors
    /// [`GroupSymbol::resolve_layout`](crate::score::visual::group_symbol::GroupSymbol).
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        self.color = params
            .user_layout
            .foreground_color
            .unwrap_or(params.app_defaults.foreground_color);
        self.font_size = params.group_name_size();
        self.padding = params.group_name_padding();
    }

    /// Sizes the name against a run of staves: `available.y` is how tall that
    /// run is, the same span a [`GroupSymbol`](crate::score::visual::group_symbol::GroupSymbol)
    /// is measured with. `available.x` is ignored -- the box's width follows
    /// from where the symbol ended up, not from a width granted here.
    pub fn measure(&mut self, available: &XY) {
        self.span = available.y.max(0.);
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
    pub fn arrange_between(&mut self, top: XY, right: f32, margin_left: f32) {
        let box_right = right - self.padding;
        let box_left = margin_left.min(box_right);
        self.bounds = BoundingBox {
            xy: XY {
                x: box_left,
                y: top.y,
            },
            size: XY {
                x: box_right - box_left,
                y: self.span,
            },
        };
    }
}
