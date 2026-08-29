//! Assembles the per-page content streams from [`PdfPageCanvas`](super::PdfPageCanvas)
//! into one PDF file, embedding every font the pages actually used a single time
//! as a Type0 / CIDFontType0 composite font (Identity encoding, CID == GID).

use std::collections::{BTreeMap, BTreeSet};

use pdf_writer::types::{CidFontType, FontFlags, SystemInfo};
use pdf_writer::{Finish, Name, Pdf, Rect, Ref, Str};
use ttf_parser::{Face, GlyphId};

use super::{EmbeddedFont, FontSet, font_resource_name, gs_fill_name, gs_stroke_name};

/// One rendered page: a media box `[x0, y0, x1, y1]` in points and a finished
/// PDF content stream, plus the glyph ids (keyed by [`FontSet`] index) and alpha
/// values it referenced so [`write_pdf`] can emit matching font-width and
/// `ExtGState` resources.
pub struct PdfPage {
    pub media_box: [f32; 4],
    pub content: Vec<u8>,
    pub used_glyphs: BTreeMap<usize, BTreeSet<u16>>,
    pub used_alphas: BTreeSet<u16>,
}

/// PDF object ids for one embedded font.
struct FontRefs {
    type0: Ref,
    cid: Ref,
    descriptor: Ref,
    font_file: Ref,
}

/// Serialises `pages` into a single PDF, embedding every font in `fonts` that
/// the pages referenced (subset to the glyph ids they used).
pub fn write_pdf(pages: &[PdfPage], fonts: &FontSet<'_>) -> Vec<u8> {
    // Glyph ids used per font index, unioned across every page.
    let mut used_glyphs: BTreeMap<usize, BTreeSet<u16>> = BTreeMap::new();
    for page in pages {
        for (&font_index, gids) in &page.used_glyphs {
            used_glyphs.entry(font_index).or_default().extend(gids);
        }
    }
    let used_alphas: BTreeSet<u16> = pages
        .iter()
        .flat_map(|p| p.used_alphas.iter().copied())
        .collect();

    let mut alloc = Ref::new(1);
    let catalog_id = alloc.bump();
    let page_tree_id = alloc.bump();

    let font_refs: BTreeMap<usize, FontRefs> = used_glyphs
        .keys()
        .map(|&font_index| {
            (
                font_index,
                FontRefs {
                    type0: alloc.bump(),
                    cid: alloc.bump(),
                    descriptor: alloc.bump(),
                    font_file: alloc.bump(),
                },
            )
        })
        .collect();

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
        {
            let mut font_dict = resources.fonts();
            for (&font_index, refs) in &font_refs {
                font_dict.pair(Name(font_resource_name(font_index).as_bytes()), refs.type0);
            }
        }
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

    for (&font_index, refs) in &font_refs {
        write_font(
            &mut pdf,
            fonts.get(font_index),
            &used_glyphs[&font_index],
            refs,
        );
    }

    pdf.finish()
}

fn write_font(
    pdf: &mut Pdf,
    font: &EmbeddedFont<'_>,
    used_glyphs: &BTreeSet<u16>,
    refs: &FontRefs,
) {
    let face: &Face = &font.face;
    let to_pdf_glyph_space = 1000.0 / f32::from(face.units_per_em());

    let base_font = Name(font.base_font.as_bytes());
    let system_info = SystemInfo {
        registry: Str(b"Adobe"),
        ordering: Str(b"Identity"),
        supplement: 0,
    };

    pdf.type0_font(refs.type0)
        .base_font(base_font)
        .encoding_predefined(Name(b"Identity-H"))
        .descendant_font(refs.cid);

    {
        let mut cid_font = pdf.cid_font(refs.cid);
        cid_font
            .subtype(CidFontType::Type0)
            .base_font(base_font)
            .system_info(system_info)
            .font_descriptor(refs.descriptor)
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
    // TODO: `SYMBOLIC` is right for the SMuFL music font (custom glyph repertoire,
    // no standard encoding). A real text font embedded here should instead be
    // flagged `NON_SYMBOLIC` (plus `SERIF` / `ITALIC` as applicable) -- revisit
    // when the first non-music font is actually added to a `FontSet`.
    pdf.font_descriptor(refs.descriptor)
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
        .font_file3(refs.font_file);

    // OpenType/CFF font program. `/Subtype /OpenType` is what tells the reader
    // the stream is a full sfnt wrapper rather than bare CFF.
    let mut stream = pdf.stream(refs.font_file, font.program);
    stream.pair(Name(b"Subtype"), Name(b"OpenType"));
}
