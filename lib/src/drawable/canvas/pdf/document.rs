//! Assembles the per-page content streams from [`PdfPageCanvas`](super::PdfPageCanvas)
//! into one PDF file, with the music font embedded a single time as a Type0 /
//! CIDFontType0 composite font (Identity encoding, CID == GID).

use std::collections::BTreeSet;

use pdf_writer::types::{CidFontType, FontFlags, SystemInfo};
use pdf_writer::{Finish, Name, Pdf, Rect, Ref, Str};
use ttf_parser::{Face, GlyphId};

use super::{FONT_NAME, gs_fill_name, gs_stroke_name};

/// One rendered page: a media box `[x0, y0, x1, y1]` in points and a finished
/// PDF content stream, plus the glyph ids and alpha values it referenced so
/// [`write_pdf`] can emit matching font-width and `ExtGState` resources.
pub struct PdfPage {
    pub media_box: [f32; 4],
    pub content: Vec<u8>,
    pub used_glyphs: BTreeSet<u16>,
    pub used_alphas: BTreeSet<u16>,
}

/// Serialises `pages` into a single PDF, embedding `font_otf` (the raw bytes of
/// the same OpenType music font the pages were measured against) once.
///
/// Panics if `font_otf` is not parseable OpenType -- it is a build asset, so a
/// failure here is a packaging bug, matching `SmuflFont::load`'s `expect`.
pub fn write_pdf(pages: &[PdfPage], font_otf: &[u8]) -> Vec<u8> {
    let face = Face::parse(font_otf, 0).expect("embed font: not valid OpenType data");
    let units_per_em = f32::from(face.units_per_em());
    let to_pdf_glyph_space = 1000.0 / units_per_em;

    let used_glyphs: BTreeSet<u16> = pages
        .iter()
        .flat_map(|p| p.used_glyphs.iter().copied())
        .collect();
    let used_alphas: BTreeSet<u16> = pages
        .iter()
        .flat_map(|p| p.used_alphas.iter().copied())
        .collect();

    let mut alloc = Ref::new(1);
    let catalog_id = alloc.bump();
    let page_tree_id = alloc.bump();
    let type0_font_id = alloc.bump();
    let cid_font_id = alloc.bump();
    let descriptor_id = alloc.bump();
    let font_file_id = alloc.bump();

    // One ExtGState object per distinct alpha, for fill and for stroke.
    let alpha_states: Vec<(u16, Ref, Ref)> = used_alphas
        .iter()
        .map(|&permille| (permille, alloc.bump(), alloc.bump()))
        .collect();

    let page_ids: Vec<Ref> = pages.iter().map(|_| alloc.bump()).collect();
    let content_ids: Vec<Ref> = pages.iter().map(|_| alloc.bump()).collect();

    let mut pdf = Pdf::new();

    pdf.catalog(catalog_id).pages(page_tree_id);
    pdf.pages(page_tree_id)
        .kids(page_ids.iter().copied())
        .count(page_ids.len() as i32);

    for (i, page) in pages.iter().enumerate() {
        let [x0, y0, x1, y1] = page.media_box;
        let mut writer = pdf.page(page_ids[i]);
        writer
            .parent(page_tree_id)
            .media_box(Rect::new(x0, y0, x1, y1))
            .contents(content_ids[i]);

        let mut resources = writer.resources();
        resources.fonts().pair(FONT_NAME, type0_font_id);
        {
            let mut ext = resources.ext_g_states();
            for &(permille, fill_id, stroke_id) in &alpha_states {
                ext.pair(Name(gs_fill_name(permille).as_bytes()), fill_id);
                ext.pair(Name(gs_stroke_name(permille).as_bytes()), stroke_id);
            }
        }
        resources.finish();
        writer.finish();

        pdf.stream(content_ids[i], &page.content);
    }

    for &(permille, fill_id, stroke_id) in &alpha_states {
        let alpha = f32::from(permille) / 1000.0;
        pdf.ext_graphics(fill_id).non_stroking_alpha(alpha);
        pdf.ext_graphics(stroke_id).stroking_alpha(alpha);
    }

    write_font(
        &mut pdf,
        &face,
        &used_glyphs,
        to_pdf_glyph_space,
        font_otf,
        type0_font_id,
        cid_font_id,
        descriptor_id,
        font_file_id,
    );

    pdf.finish()
}

#[allow(clippy::too_many_arguments)]
fn write_font(
    pdf: &mut Pdf,
    face: &Face,
    used_glyphs: &BTreeSet<u16>,
    to_pdf_glyph_space: f32,
    font_otf: &[u8],
    type0_font_id: Ref,
    cid_font_id: Ref,
    descriptor_id: Ref,
    font_file_id: Ref,
) {
    let base_font = Name(b"Bravura");
    let system_info = SystemInfo {
        registry: Str(b"Adobe"),
        ordering: Str(b"Identity"),
        supplement: 0,
    };

    pdf.type0_font(type0_font_id)
        .base_font(base_font)
        .encoding_predefined(Name(b"Identity-H"))
        .descendant_font(cid_font_id);

    {
        let mut cid_font = pdf.cid_font(cid_font_id);
        cid_font
            .subtype(CidFontType::Type0)
            .base_font(base_font)
            .system_info(system_info)
            .font_descriptor(descriptor_id)
            .cid_to_gid_map_predefined(Name(b"Identity"))
            .default_width(0.0);

        let mut widths = cid_font.widths();
        for &gid in used_glyphs {
            let advance = f32::from(face.glyph_hor_advance(GlyphId(gid)).unwrap_or(0));
            widths.same(gid, gid, advance * to_pdf_glyph_space);
        }
    }

    let bbox = face.global_bounding_box();
    let scale = |v: i16| f32::from(v) * to_pdf_glyph_space;
    pdf.font_descriptor(descriptor_id)
        .name(base_font)
        .flags(FontFlags::SYMBOLIC)
        .bbox(Rect::new(
            scale(bbox.x_min),
            scale(bbox.y_min),
            scale(bbox.x_max),
            scale(bbox.y_max),
        ))
        .italic_angle(0.0)
        .ascent(scale(face.ascender()))
        .descent(scale(face.descender()))
        .cap_height(scale(face.ascender()))
        .stem_v(80.0)
        .font_file3(font_file_id);

    // OpenType/CFF font program. `/Subtype /OpenType` is what tells the reader
    // the stream is a full sfnt wrapper rather than bare CFF.
    let mut stream = pdf.stream(font_file_id, font_otf);
    stream.pair(Name(b"Subtype"), Name(b"OpenType"));
}
