mod base_renderer;
mod debug_renderer;

pub use base_renderer::BaseRenderer;
pub use debug_renderer::DebugRenderer;

use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::elements::polygon::Polygon;
use crate::score::visual::render_fonts::RenderFonts;
use crate::score::visual::{
    accidental::Accidental, chord::Chord, clef::Clef, dot::Dot, flag::Flag, group_name::GroupName,
    group_symbol::GroupSymbol, key_signature::KeySignature, note::Note, page::Page, part::Part,
    part_group::PartGroup, part_group_measure::PartGroupMeasure, part_measure::PartMeasure,
    rest::Rest, section::Section, section_measure::SectionMeasure, staff::Staff,
    staff_measure::StaffMeasure, stem::Stem, system::System, system_measure::SystemMeasure,
    tie::TieSegment, time_signature::TimeSignature,
};

pub trait RenderPass {
    fn render_page<'a>(
        &self,
        _page: &Page,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_system<'a>(
        &self,
        _system: &System,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_section<'a>(
        &self,
        _section: &Section,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    /// The one entry point for all five group-symbol shapes, at all three
    /// levels: a section's, a part-group's and a part's are the same element and
    /// are drawn the same way.
    fn render_group_symbol<'a>(
        &self,
        _symbol: &GroupSymbol,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    /// The one entry point for a part's and a part-group's name alike, the way
    /// [`render_group_symbol`](Self::render_group_symbol) serves both. `name`
    /// borrows for `'a` because the run it draws is a slice of the score tree,
    /// not of the font.
    fn render_group_name<'a>(
        &self,
        _name: &'a GroupName,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_part_group<'a>(
        &self,
        _part_group: &PartGroup,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_part<'a>(
        &self,
        _part: &Part,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_staff<'a>(
        &self,
        _staff: &Staff,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_system_measure<'a>(
        &self,
        _system_measure: &SystemMeasure,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_section_measure<'a>(
        &self,
        _section_measure: &SectionMeasure,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_part_group_measure<'a>(
        &self,
        _part_group_measure: &PartGroupMeasure,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_part_measure<'a>(
        &self,
        _part_measure: &PartMeasure,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_staff_measure<'a>(
        &self,
        _staff_measure: &StaffMeasure,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_clef<'a>(
        &self,
        _clef: &Clef,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_key_signature<'a>(
        &self,
        _key_signature: &KeySignature,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_accidental<'a>(
        &self,
        _accidental: &Accidental,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_time_signature<'a>(
        &self,
        _time_signature: &TimeSignature,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_rest<'a>(
        &self,
        _rest: &Rest,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_chord<'a>(
        &self,
        _chord: &Chord,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_note<'a>(
        &self,
        _note: &Note,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_stem<'a>(
        &self,
        _stem: &Stem,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_flag<'a>(
        &self,
        _flag: &Flag,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_dot<'a>(
        &self,
        _dot: &Dot,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    /// One segment of one beam. Takes the finished quad rather than a score
    /// element because that is all a beam is: `arrange_beams` resolves the whole
    /// group's geometry and files the segments on the `Part`, so there is no
    /// `Beam` in the tree for a pass to be handed.
    fn render_beam<'a>(
        &self,
        _beam: &Polygon,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }

    fn render_tie<'a>(
        &self,
        _tie: &TieSegment,
        _fonts: &RenderFonts<'a>,
        _out: &mut Vec<DrawableElement<'a>>,
    ) {
    }
}
