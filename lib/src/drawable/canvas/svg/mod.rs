use crate::drawable::canvas::Canvas;
use crate::drawable::elements::line::Line;
use crate::drawable::elements::polygon::Polygon;
use crate::drawable::elements::rect::Rect;
use crate::drawable::elements::text::Text;

/// A [`Canvas`] that renders elements into an SVG document string.
#[derive(Default)]
pub struct SvgCanvas {
    out: String,
}

impl SvgCanvas {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Canvas for SvgCanvas {
    type Output = String;

    fn begin(&mut self, bounds: (f32, f32, f32, f32)) {
        let (min_x, min_y, max_x, max_y) = bounds;
        let width = max_x - min_x;
        let height = max_y - min_y;

        self.out.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{min_x} {min_y} {w} {h}" width="{w}" height="{h}">"#,
            min_x = min_x,
            min_y = min_y,
            w = width,
            h = height,
        ));
        self.out.push_str("\r\n");
    }

    fn draw_line(&mut self, l: &Line) {
        self.out.push_str(&format!(
            r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{}" stroke-width="{}" />"#,
            l.stroke_color.to_hex(),
            l.stroke_width,
            x1 = l.start.x,
            y1 = l.start.y,
            x2 = l.end.x,
            y2 = l.end.y,
        ));
        self.out.push_str("\r\n");
    }

    fn draw_rect(&mut self, r: &Rect) {
        self.out.push_str(&format!(
            r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{}" stroke="{}" stroke-width="{}" />"#,
            r.color.to_hex(),
            r.stroke_color.map_or("none".to_string(), |s| s.to_hex()),
            r.stroke_width.unwrap_or(0.0),
            x = r.xy.x,
            y = r.xy.y,
            w = r.width,
            h = r.height,
        ));
        self.out.push_str("\r\n");
    }

    fn draw_text(&mut self, t: &Text<'_>) {
        self.out.push_str(&format!(
            r#"<text x="{x}" y="{y}" fill="{fill}" font-size="{fs}" font-family="{ff}" font-weight="{fw}" font-style="{fst}" text-anchor="{ha}" dominant-baseline="{va}">{content}</text>"#,
            x = t.xy.x,
            y = t.xy.y,
            fill = t.color.to_hex(),
            fs = t.font_size,
            ff = xml_escape(t.font.family),
            fw = t.font.weight.to_css(),
            fst = t.font.style.to_css(),
            ha = t.horizontal_alignment.to_svg(),
            va = t.vertical_alignment.to_svg(),
            content = xml_escape(t.text),
        ));
        self.out.push_str("\r\n");
    }

    fn draw_polygon(&mut self, p: &Polygon) {
        let pts = p
            .pts
            .iter()
            .map(|xy| format!("{},{}", xy.x, xy.y))
            .collect::<Vec<_>>()
            .join(" ");

        self.out.push_str(&format!(
            r#"<polygon points="{pts}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}" />"#,
            pts = pts,
            fill = p.color.to_hex(),
            stroke = p
                .stroke_color
                .as_ref()
                .map_or("none".to_string(), |c| c.to_hex()),
            sw = p.stroke_width.unwrap_or(0.0),
        ));
        self.out.push_str("\r\n");
    }

    fn finish(mut self) -> String {
        self.out.push_str("</svg>");
        self.out
    }
}

// Very small XML escape helper
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
