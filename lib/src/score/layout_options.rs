//! The layout option set, declared once.
//!
//! Every option exists twice: as an override on [`UserLayout`], where it is
//! always optional, and as a fallback on [`AppDefaults`], where it always has a
//! value. Both are generated from the single `layout_options!` invocation
//! below, so an option cannot exist on one and not the other -- adding one is a
//! single line here, and every front end (the CLI's flags and layout file, the
//! wasm bindings) picks it up from `UserLayout`.
//!
//! Options are nested by concern: `tie.height_max`, `section.symbol`. Their flat
//! spelling -- a CLI flag, a CSS custom property -- is the kebab-case of that
//! path: `--tie-height-max`, `--section-symbol`.

use crate::geometry::color::Color;
use crate::score::core::group_symbol::GroupSymbol;
use serde::{Deserialize, Serialize};

layout_options! {
    /// The page's background.
    page: PageLayout, PageDefaults {
        color: Color = Color::WHITE,
    }

    /// The ink everything on the page is drawn in.
    foreground: ForegroundLayout, ForegroundDefaults {
        color: Color = Color::BLACK,
    }

    staff: StaffLayout, StaffDefaults {
        /// Thickness in tenths of a staff line, and of a ledger line.
        line_width: f32 = 1.1,
    }

    /// Barline thicknesses in tenths.
    barline: BarlineLayout, BarlineDefaults {
        light: f32 = 1.875,
        heavy: f32 = 5.,
    }

    /// Beam geometry in tenths.
    beam: BeamLayout, BeamDefaults {
        thickness: f32 = 5.,
        spacing: f32 = 1.5,
    }

    stem: StemLayout, StemDefaults {
        /// Stem thickness in tenths.
        thickness: f32 = 1.,
    }

    /// Fraction of full size a grace or cue note is drawn at, overriding what
    /// the document declared in `<defaults><appearance><note-size>`. `0.5` is
    /// half size; `1.` draws them like any other note.
    ///
    /// Applies to everything the note owns -- notehead, dots, stem, flag and the
    /// beams joining it to its group -- because all of them resolve through
    /// [`LayoutParams::note_size`](crate::score::visual::layoutable::LayoutParams::note_size).
    note_size: NoteSizeLayout, NoteSizeDefaults {
        grace: f32 = 0.66,
        cue: f32 = 0.66,
    }

    /// Augmentation dots.
    dot: DotLayout, DotDefaults {
        /// Augmentation-dot radius, in tenths.
        radius: f32 = 2.,
        /// Augmentation-dot spacing, in tenths: both the gap from the notehead /
        /// rest right edge to the first dot's centre and the centre-to-centre
        /// step between successive dots.
        spacing: f32 = 5.,
    }

    /// Padding at the start of a measure, in tenths. Each is scaled by the
    /// staff measure's own `scale`, and its column is settled from the widest
    /// contribution across the system's drawn staves.
    measure_start: MeasureStartLayout, MeasureStartDefaults {
        /// Between a measure's left edge and the opening clef, when the measure
        /// opens with one.
        clef_padding: f32 = 5.,
        /// Between the opening clef column and the key signature.
        key_signature_padding: f32 = 5.,
        /// Between the key signature column and the time signature, when the
        /// measure opens with one.
        time_signature_padding: f32 = 5.,
    }

    /// Tie shape, in tenths except `height_ratio`.
    tie: TieLayout, TieDefaults {
        /// Tie thickness at a notehead end. Bravura's `tieEndpointThickness` is
        /// 0.1 staff spaces; one space is
        /// [`Staff::DEFAULT_SPACE_SIZE`](crate::score::visual::staff::Staff) = 10
        /// tenths. Quoted rather than read, because `SmuflMetadata` does not
        /// deserialize `engravingDefaults` -- the same reason `beam.thickness` is
        /// hard-coded to Bravura's 0.5 spaces.
        endpoint_thickness: f32 = 1.,
        /// Tie thickness at its widest point. Bravura's `tieMidpointThickness`,
        /// 0.22 staff spaces.
        midpoint_thickness: f32 = 2.2,
        /// A tie's arc height as a fraction of its horizontal span, before
        /// clamping.
        height_ratio: f32 = 0.15,
        /// Lower clamp on a tie's arc height -- so a very short tie still reads
        /// as a curve rather than a smear.
        height_min: f32 = 5.,
        /// Upper clamp on a tie's arc height.
        height_max: f32 = 16.,
        /// Gap between a notehead's edge and the tie tip that meets it.
        note_gap: f32 = 2.,
        /// Offset from a notehead's vertical centre to the tie tip.
        vertical_offset: f32 = 5.,
        /// Margin between the barline and the far end of the opening fragment
        /// of a tie broken across a system break. That fragment runs out to the
        /// end of its own measure, less this.
        break_inset: f32 = 10.,
        /// Length of the closing fragment of a broken tie -- the short
        /// "courtesy" arc that arrives at the note on the next system. Also the
        /// minimum length of the opening fragment, for a note sitting right at
        /// the end of its measure.
        break_fragment: f32 = 20.,
    }

    /// What binds a section's part-groups together, and where it sits.
    section: SectionLayout, SectionDefaults {
        /// The symbol drawn when the document's `<part-group>` names no
        /// `<group-symbol>`. As an override, it replaces whatever every section
        /// declared; [`GroupSymbol::None`] suppresses them entirely.
        symbol: GroupSymbol = GroupSymbol::Bracket,
        /// Distance in tenths from the system's left edge to the right edge of
        /// a section symbol's vertical stroke.
        ///
        /// A section's symbol is the innermost of the three, so this one is
        /// measured from the system itself. The other two are measured from
        /// whatever is already there.
        ///
        /// A bracket's tips flare rightward past its own stroke -- that is how
        /// SMuFL draws them -- so its ink reaches nearer the staff than the gap
        /// suggests, and may cross it.
        symbol_gap: f32 = 5.,
    }

    /// What binds a part-group's parts together, and where it sits.
    part_group: PartGroupLayout, PartGroupDefaults {
        /// The symbol drawn absent a `<group-symbol>`; as an override, forced on
        /// every part-group.
        symbol: GroupSymbol = GroupSymbol::Brace,
        /// Padding in tenths between the left edge of what a section drew and
        /// the right edge of a part-group's symbol inside it.
        ///
        /// Padding rather than a distance from the system, because a fixed
        /// ladder of distances cannot hold: a brace's width follows the span it
        /// covers, so a tall part-group's symbol is far wider than a short one's
        /// and would eventually reach under the section symbol outside it.
        symbol_gap: f32 = 2.,
    }

    /// What binds one part's own staves together, and where it sits.
    part: PartLayout, PartDefaults {
        /// MusicXML declares this in `<attributes><part-symbol>`, which nothing
        /// reads yet, so today this is always what a multi-staff part gets.
        ///
        /// Nothing stops a part being given a `Bracket`: its tips reach past the
        /// system's left edge and would sit over the clef. That is the caller's
        /// choice, not a case to guard against.
        symbol: GroupSymbol = GroupSymbol::Brace,
        /// Padding in tenths between the left edge of what a part-group drew and
        /// the right edge of the symbol joining one part's own staves. As
        /// `part_group.symbol_gap`, measured from what is already there.
        symbol_gap: f32 = 5.,
    }

    group_bracket: GroupBracketLayout, GroupBracketDefaults {
        /// Thickness in tenths of a bracket's vertical stroke. Bravura's
        /// `engravingDefaults.bracketThickness` is 0.5 staff spaces. Quoted
        /// rather than read, for the reason the `tie` values are.
        ///
        /// The bracket's tip glyphs are scaled to match, so a thicker stroke
        /// keeps serifs in proportion to it.
        thickness: f32 = 5.,
    }

    group_line: GroupLineLayout, GroupLineDefaults {
        /// Thickness in tenths of a `line` symbol. Bravura's
        /// `subBracketThickness`, 0.16 staff spaces -- the weight SMuFL documents
        /// for the vertical line grouping staves of one instrument, which is
        /// what `line` is for.
        thickness: f32 = 1.6,
    }

    /// SMuFL defines no glyph or figure of its own for a square, so unlike the
    /// bracket's and the line's these are quoted from nothing: tuned by eye.
    group_square: GroupSquareLayout, GroupSquareDefaults {
        /// Thickness in tenths of a `square` symbol's stroke, spine and arms
        /// alike. Deliberately lighter than a bracket's.
        thickness: f32 = 1.2,
        /// How far in tenths a `square` symbol's arms reach toward the system,
        /// measured from the right edge of its spine.
        arm: f32 = 8.,
    }

    /// Part and part-group names.
    group_name: GroupNameLayout, GroupNameDefaults {
        /// Font family. A generic CSS family so both a browser and a system
        /// font database can resolve it.
        font: String as &'static str = "serif",
        /// Font size in tenths. About 1.6 staff spaces, a staff space being 10
        /// tenths.
        size: f32 = 16.,
        /// Padding in tenths between a name's right edge and the symbol it sits
        /// beside. One staff space.
        padding: f32 = 10.,
    }

    /// Titles and work-level text.
    title: TitleLayout, TitleDefaults {
        /// Font family, resolved like `group_name.font`.
        font: String as &'static str = "serif",
    }

    /// Lyrics.
    lyric: LyricLayout, LyricDefaults {
        /// Font family, resolved like `group_name.font`.
        font: String as &'static str = "serif",
    }
}

/// Generates, from one list of groups, [`UserLayout`], [`AppDefaults`],
/// [`APP_DEFAULTS`] and one pair of structs per group.
///
/// A field is `name: Type = default`. `name: Type as AppType = default` gives
/// the fallback a different type from the override -- a font family is an
/// owned `String` when a caller supplies it but a `&'static str` in the const.
macro_rules! layout_options {
    ($(
        $(#[$group_attr:meta])*
        $group:ident: $layout:ident, $defaults:ident {
            $(
                $(#[$field_attr:meta])*
                $field:ident: $ty:ty $(as $app_ty:ty)? = $default:expr
            ),* $(,)?
        }
    )*) => {
        /// Caller-supplied overrides, each falling back to [`APP_DEFAULTS`]
        /// when `None`.
        ///
        /// This struct is the single source of truth for the layout option set
        /// a front end sees: it deserializes directly (snake_case, every field
        /// optional, unknown keys rejected) from a CLI layout file or the wasm
        /// `layout` object. Don't mirror it into a parallel options struct
        /// somewhere else; that only creates two lists to keep in step.
        ///
        /// `Serialize` is derived only so a test can enumerate its field names
        /// by serializing a default instance; nothing serializes a real
        /// `UserLayout`.
        #[derive(Clone, Debug, Default, Deserialize, Serialize)]
        #[serde(default, deny_unknown_fields)]
        pub struct UserLayout {
            $(
                $(#[$group_attr])*
                pub $group: $layout,
            )*
        }

        impl UserLayout {
            /// `overrides` layered over `self`: every option `overrides` sets
            /// wins, every one it leaves `None` keeps the value `self` had.
            /// How the CLI puts its flags on top of a layout file.
            pub fn overlay(self, overrides: UserLayout) -> UserLayout {
                UserLayout {
                    $($group: self.$group.overlay(overrides.$group),)*
                }
            }
        }

        /// The hard-coded fallback for every [`UserLayout`] option, read from
        /// the one [`APP_DEFAULTS`] const.
        #[derive(Debug)]
        pub struct AppDefaults {
            $(
                $(#[$group_attr])*
                pub $group: $defaults,
            )*
        }

        pub const APP_DEFAULTS: AppDefaults = AppDefaults {
            $($group: $defaults {
                $($field: $default,)*
            },)*
        };

        $(
            $(#[$group_attr])*
            #[derive(Clone, Debug, Default, Deserialize, Serialize)]
            #[serde(default, deny_unknown_fields)]
            pub struct $layout {
                $(
                    $(#[$field_attr])*
                    pub $field: Option<$ty>,
                )*
            }

            impl $layout {
                fn overlay(self, overrides: $layout) -> $layout {
                    $layout {
                        $($field: overrides.$field.or(self.$field),)*
                    }
                }
            }

            $(#[$group_attr])*
            #[derive(Debug)]
            pub struct $defaults {
                $(
                    $(#[$field_attr])*
                    pub $field: app_type!($ty $(as $app_ty)?),
                )*
            }
        )*
    };
}

/// The fallback's type for a [`layout_options!`] field: the `as` type when one
/// is given, the override's own otherwise.
macro_rules! app_type {
    ($ty:ty as $app_ty:ty) => {
        $app_ty
    };
    ($ty:ty) => {
        $ty
    };
}

use {app_type, layout_options};
