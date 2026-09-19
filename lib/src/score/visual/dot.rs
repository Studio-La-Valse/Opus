use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::note_scale::NoteScale;

/// A single augmentation dot belonging to a [`Note`](super::note::Note) or
/// [`Rest`](super::rest::Rest). Notes and rests own a `Vec<Dot>` with one entry
/// per dot; the owner computes each dot's centre during its arrange pass
/// (horizontal spacing, plus the half-space vertical nudge that keeps a dot off
/// a staff line) and hands it here through
/// [`arrange_dot`](crate::score::visual::arrange_machine::ArrangeMachine::arrange_dot).
pub struct Dot {
    pub xy: XY,

    /// What this dot's size is derived from -- the owning note's own
    /// [`NoteScale`], so a grace note's dots shrink with it.
    pub size: NoteScale,
    /// The factor `size` resolved to, written by `resolve_layout` and so only
    /// meaningful once that pass has run.
    pub scale: f32,

    pub color: Color,

    /// Resolved dot radius in world units (tenths).
    pub radius: f32,
}

impl Dot {
    pub fn new(size: NoteScale) -> Self {
        Self {
            xy: XY::ZERO,

            size,
            scale: size.content_scale,

            color: Color::BLACK,

            radius: 0.,
        }
    }
}

impl Dot {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            user_layout,
            app_defaults,
            ..
        } = params;

        self.scale = self.size.resolve(params);

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
        self.radius = user_layout.dot_radius.unwrap_or(app_defaults.dot_radius) * self.scale;
    }
}
