#[cfg(test)]
mod tests {
    use lib::drawable::canvas::CanvasPainter;
    use lib::drawable::canvas::flat_buffer::{
        FlatBufferCanvas, TAG_CIRCLE, TAG_LINE, TAG_POLYGON, TAG_RECT, TAG_TEXT,
    };
    use lib::drawable::drawable_element::DrawableElement;
    use lib::drawable::elements::circle::Circle;
    use lib::drawable::elements::line::Line;
    use lib::drawable::elements::polygon::Polygon;
    use lib::drawable::elements::rect::Rect;
    use lib::drawable::elements::text::{FontSpec, HorizontalAlign, Text, VerticalAlign};
    use lib::geometry::color::Color;
    use lib::geometry::xy::XY;
    use lib::score::visual::render_compositor::RenderedPage;

    #[test]
    fn line_record_layout() {
        let elements: Vec<DrawableElement<'_>> = vec![
            Line {
                start: XY { x: 1.0, y: 2.0 },
                end: XY { x: 3.0, y: 4.0 },
                stroke_color: Color::RED,
                stroke_width: 0.5,
            }
            .into(),
        ];

        let flat = CanvasPainter::new(FlatBufferCanvas::new()).paint(&elements);

        assert_eq!(
            flat.geometry,
            vec![TAG_LINE, 1.0, 2.0, 3.0, 4.0, 255.0, 0.0, 0.0, 1.0, 0.5]
        );
        assert!(flat.text_blob.is_empty());
    }

    #[test]
    fn rect_record_layout_with_and_without_stroke() {
        let elements: Vec<DrawableElement<'_>> = vec![
            Rect {
                xy: XY { x: 0.0, y: 0.0 },
                width: 10.0,
                height: 20.0,
                color: Color::WHITE,
                stroke_color: None,
                stroke_width: None,
            }
            .into(),
        ];

        let flat = CanvasPainter::new(FlatBufferCanvas::new()).paint(&elements);

        assert_eq!(
            flat.geometry,
            vec![
                TAG_RECT, 0.0, 0.0, 10.0, 20.0, 255.0, 255.0, 255.0, 1.0, -1.0, 0.0, 0.0, 0.0, 0.0,
            ]
        );
    }

    #[test]
    fn circle_record_layout_with_and_without_stroke() {
        let elements: Vec<DrawableElement<'_>> = vec![
            Circle {
                xy: XY { x: 3.0, y: 4.0 },
                radius: 5.0,
                color: Color::WHITE,
                stroke_color: None,
                stroke_width: None,
            }
            .into(),
            Circle {
                xy: XY { x: 1.0, y: 2.0 },
                radius: 6.0,
                color: Color::WHITE,
                stroke_color: Some(Color::RED),
                stroke_width: Some(0.5),
            }
            .into(),
        ];

        let flat = CanvasPainter::new(FlatBufferCanvas::new()).paint(&elements);

        assert_eq!(
            flat.geometry,
            vec![
                TAG_CIRCLE, 3.0, 4.0, 5.0, 255.0, 255.0, 255.0, 1.0, -1.0, 0.0, 0.0, 0.0, 0.0,
                TAG_CIRCLE, 1.0, 2.0, 6.0, 255.0, 255.0, 255.0, 1.0, 0.5, 255.0, 0.0, 0.0, 1.0,
            ]
        );
    }

    #[test]
    fn text_record_layout_and_ordering() {
        let elements: Vec<DrawableElement<'_>> = vec![
            Text {
                text: "a",
                color: Color::BLACK,
                font_size: 12.0,
                font: FontSpec::plain("Bravura"),
                xy: XY { x: 5.0, y: 6.0 },
                vertical_alignment: VerticalAlign::Middle,
                horizontal_alignment: HorizontalAlign::Center,
            }
            .into(),
            Text {
                text: "bb",
                color: Color::BLACK,
                font_size: 12.0,
                font: FontSpec::plain("Bravura"),
                xy: XY { x: 7.0, y: 8.0 },
                vertical_alignment: VerticalAlign::Top,
                horizontal_alignment: HorizontalAlign::Left,
            }
            .into(),
        ];

        let flat = CanvasPainter::new(FlatBufferCanvas::new()).paint(&elements);

        assert_eq!(
            flat.geometry,
            vec![
                // trailing 0.0 on each record is the font index into `font_blob`
                TAG_TEXT, 5.0, 6.0, 12.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0,
                0.0, // "a": center/middle
                TAG_TEXT, 7.0, 8.0, 12.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, // "bb": left/top
            ]
        );
        assert_eq!(
            flat.text_blob,
            format!("a{}bb", lib::drawable::canvas::flat_buffer::TEXT_DELIMITER)
        );
        // Both records use the one font, so a single deduped entry.
        assert_eq!(flat.font_blob, "Bravura");
        assert_eq!(flat.font_styles, vec![0.0]);
    }

    #[test]
    fn distinct_fonts_are_pooled_and_indexed() {
        use lib::drawable::canvas::flat_buffer::{
            FONT_STYLE_BOLD, FONT_STYLE_ITALIC, TEXT_DELIMITER,
        };
        use lib::drawable::elements::text::{FontStyle, FontWeight};

        let bravura = FontSpec::plain("Bravura");
        let title_bold = FontSpec {
            family: "serif",
            weight: FontWeight::Bold,
            style: FontStyle::Normal,
        };
        let lyric_italic = FontSpec {
            family: "serif",
            weight: FontWeight::Normal,
            style: FontStyle::Italic,
        };

        let text = |font| {
            DrawableElement::from(Text {
                text: "x",
                color: Color::BLACK,
                font_size: 10.0,
                font,
                xy: XY { x: 0.0, y: 0.0 },
                vertical_alignment: VerticalAlign::Top,
                horizontal_alignment: HorizontalAlign::Left,
            })
        };
        // bravura, then title, then lyric, then title again (should reuse idx 1).
        let elements = vec![
            text(bravura),
            text(title_bold),
            text(lyric_italic),
            text(title_bold),
        ];

        let flat = CanvasPainter::new(FlatBufferCanvas::new()).paint(&elements);

        // Every record is 11 f32s; the last is the font index.
        let indices: Vec<f32> = flat.geometry.chunks(11).map(|r| r[10]).collect();
        assert_eq!(indices, vec![0.0, 1.0, 2.0, 1.0]);
        assert_eq!(
            flat.font_blob,
            format!("Bravura{d}serif{d}serif", d = TEXT_DELIMITER)
        );
        assert_eq!(
            flat.font_styles,
            vec![0.0, FONT_STYLE_BOLD as f32, FONT_STYLE_ITALIC as f32,]
        );
    }

    #[test]
    fn paint_pages_records_a_page_table_parallel_to_the_concatenated_geometry() {
        let line = |x: f32| {
            DrawableElement::from(Line {
                start: XY { x, y: 0.0 },
                end: XY { x: x + 2.0, y: 0.0 },
                stroke_color: Color::RED,
                stroke_width: 0.5,
            })
        };
        let rect = || {
            DrawableElement::from(Rect {
                xy: XY { x: 0.0, y: 0.0 },
                width: 4.0,
                height: 4.0,
                color: Color::WHITE,
                stroke_color: None,
                stroke_width: None,
            })
        };

        let pages = [
            RenderedPage {
                number: 1,
                origin: XY { x: 0.0, y: 0.0 },
                width: 100.0,
                height: 50.0,
                elements: vec![line(0.0)],
            },
            RenderedPage {
                number: 2,
                origin: XY { x: 100.0, y: 0.0 },
                width: 100.0,
                height: 50.0,
                elements: vec![line(100.0), rect()],
            },
        ];

        let flat = CanvasPainter::new(FlatBufferCanvas::new()).paint_pages(&pages);

        // Geometry is the pages' records concatenated in page order: one 10-f32
        // line for page 0, then a line + a 14-f32 rect for page 1.
        assert_eq!(flat.geometry.len(), 10 + 10 + 14);
        assert_eq!(&flat.geometry[..10], &line_record(0.0));
        assert_eq!(&flat.geometry[10..20], &line_record(100.0));

        // page_table: [start_index, origin_x, origin_y, width, height] per page.
        // Page 0 starts at 0; page 1 starts after page 0's 10 f32s.
        assert_eq!(
            flat.page_table,
            vec![0.0, 0.0, 0.0, 100.0, 50.0, 10.0, 100.0, 0.0, 100.0, 50.0]
        );
    }

    fn line_record(x: f32) -> [f32; 10] {
        [TAG_LINE, x, 0.0, x + 2.0, 0.0, 255.0, 0.0, 0.0, 1.0, 0.5]
    }

    #[test]
    fn paint_leaves_the_page_table_empty() {
        let elements: Vec<DrawableElement<'_>> = vec![
            Line {
                start: XY { x: 0.0, y: 0.0 },
                end: XY { x: 1.0, y: 1.0 },
                stroke_color: Color::RED,
                stroke_width: 1.0,
            }
            .into(),
        ];

        let flat = CanvasPainter::new(FlatBufferCanvas::new()).paint(&elements);

        assert!(flat.page_table.is_empty());
    }

    #[test]
    fn polygon_record_layout_is_variable_length() {
        let elements: Vec<DrawableElement<'_>> = vec![
            Polygon {
                pts: vec![
                    XY { x: 0.0, y: 0.0 },
                    XY { x: 1.0, y: 0.0 },
                    XY { x: 0.5, y: 1.0 },
                ],
                color: Color::GREEN,
                stroke_color: None,
                stroke_width: None,
            }
            .into(),
        ];

        let flat = CanvasPainter::new(FlatBufferCanvas::new()).paint(&elements);

        assert_eq!(
            flat.geometry,
            vec![
                TAG_POLYGON,
                3.0,
                0.0,
                255.0,
                0.0,
                1.0,
                -1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                0.5,
                1.0,
            ]
        );
    }
}
