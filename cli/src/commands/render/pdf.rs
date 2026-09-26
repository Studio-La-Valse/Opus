use lib::drawable::canvas::CanvasPainter;
use lib::drawable::canvas::pdf::{self, EmbeddedFont, FontSet, PdfPage, PdfPageCanvas};
use lib::drawable::elements::text::{FontSpec, FontStyle, FontWeight};
use lib::score::score_defaults::Defaults;
use lib::score::visual::render_compositor::RenderedPage;
use lib::score::visual::render_fonts::RenderFonts;
use std::time::Instant;
use ttf_parser::Face;

use super::paths::OutputTarget;

/// Renders the already-walked `pages` to a single multi-page PDF (`<stem>.pdf`)
/// under `target`'s directory, with every font it draws text in embedded once.
///
/// Font sourcing is PDF-specific: the families named in `fonts` are resolved
/// against the installed system fonts and their programs subset/embedded, so
/// this writer owns a `fontdb` lookup the other formats don't need.
pub(super) fn write(
    pages: &[RenderedPage<'_>],
    fonts: &RenderFonts<'_>,
    defaults: &Defaults,
    target: &OutputTarget,
) {
    let music_family = fonts.music.family;

    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    // Own the bytes here so the parsed `Face`s and the slices `write_pdf` embeds
    // can borrow them for the rest of this function.
    let music_bytes = load_family_bytes(&db, fonts.music).unwrap_or_else(|| {
        panic!("music font '{music_family}' not found in the system fonts; install it")
    });
    let title_bytes = load_family_bytes(&db, fonts.title);
    let lyric_bytes = load_family_bytes(&db, fonts.lyric);
    let group_name_bytes = load_family_bytes(&db, fonts.group_name);

    // Embed each family the score draws text in exactly once, in a fixed order
    // so the music font is always entry 0 -- which `FontSet` uses as the
    // fallback for any family that didn't resolve (e.g. a text font that isn't
    // installed). A hand-written chain of `!=` comparisons does not scale past
    // two text faces, so this walks the list and skips families already added.
    let mut embedded: Vec<EmbeddedFont> = Vec::new();
    for (family, bytes) in [
        (music_family, Some(&music_bytes)),
        (fonts.title.family, title_bytes.as_ref()),
        (fonts.lyric.family, lyric_bytes.as_ref()),
        (fonts.group_name.family, group_name_bytes.as_ref()),
    ] {
        if embedded.iter().any(|e| e.family == family) {
            continue;
        }
        if let Some(bytes) = bytes {
            embedded.push(embedded_font(family, bytes));
        }
    }
    let font_set = FontSet::new(embedded, 0);

    // MusicXML tenths -> PDF points: the score's mm-per-tenth scaling times 72
    // points per inch over 25.4 mm per inch.
    let pt_per_tenth = defaults.scaling_millimeters / defaults.scaling_tenths * 72.0 / 25.4;

    let time = Instant::now();

    let pdf_pages: Vec<PdfPage> = pages
        .iter()
        .map(|page| {
            let canvas = PdfPageCanvas::new((page.width, page.height), pt_per_tenth, &font_set);
            CanvasPainter::new(canvas).paint(&page.elements)
        })
        .collect();

    let path = target.single("pdf");
    target.write(&path, pdf::write_pdf(&pdf_pages, &font_set));

    println!("Write to pdf: {}ms", time.elapsed().as_millis());
    println!("Written to: {}", path.display());
}

/// Copies out the raw bytes of the first installed font in `spec`'s family
/// list, at regular weight / upright style, or `None` if none is installed.
/// The CSS generic keywords resolve through fontdb's generic families rather
/// than as literal names.
fn load_family_bytes(db: &fontdb::Database, spec: FontSpec<'_>) -> Option<Vec<u8>> {
    let families: Vec<fontdb::Family> = spec
        .families()
        .map(|family| match family {
            "serif" => fontdb::Family::Serif,
            "sans-serif" => fontdb::Family::SansSerif,
            "monospace" => fontdb::Family::Monospace,
            "cursive" => fontdb::Family::Cursive,
            "fantasy" => fontdb::Family::Fantasy,
            name => fontdb::Family::Name(name),
        })
        .collect();
    let id = db.query(&fontdb::Query {
        families: &families,
        weight: fontdb::Weight::NORMAL,
        stretch: fontdb::Stretch::Normal,
        style: fontdb::Style::Normal,
    })?;
    db.with_face_data(id, |data, _index| data.to_vec())
}

/// Builds an [`EmbeddedFont`] borrowing `bytes`, deriving the PDF `BaseFont`
/// name from the font's own PostScript name (falling back to `family`).
fn embedded_font<'f>(family: &str, bytes: &'f [u8]) -> EmbeddedFont<'f> {
    let face = Face::parse(bytes, 0)
        .unwrap_or_else(|err| panic!("font '{family}' is not valid OpenType: {err}"));
    let base_font = face
        .names()
        .into_iter()
        .find(|n| n.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
        .and_then(|n| n.to_string())
        .unwrap_or_else(|| family.replace(' ', ""));

    EmbeddedFont {
        family: family.to_string(),
        weight: FontWeight::Normal,
        style: FontStyle::Normal,
        base_font,
        face,
        program: bytes,
    }
}
