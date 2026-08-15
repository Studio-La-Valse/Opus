#[cfg(test)]
mod tests {
    use lib::drawable::drawable_element::DrawableElement;
    use lib::drawable::elements::line::Line;
    use lib::drawable::elements::polygon::Polygon;
    use lib::drawable::elements::rect::Rect;
    use lib::drawable::elements::text::{HorizontalAlign, Text, VerticalAlign};
    use lib::drawable::flat_buffer::{TAG_LINE, TAG_POLYGON, TAG_RECT, TAG_TEXT, to_flat_buffer};
    use lib::geometry::color::Color;
    use lib::geometry::xy::XY;

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

        let flat = to_flat_buffer(elements);

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

        let flat = to_flat_buffer(elements);

        assert_eq!(
            flat.geometry,
            vec![
                TAG_RECT, 0.0, 0.0, 10.0, 20.0, 255.0, 255.0, 255.0, 1.0, -1.0, 0.0, 0.0, 0.0, 0.0,
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
                font: "Bravura",
                xy: XY { x: 5.0, y: 6.0 },
                vertical_alignment: VerticalAlign::Middle,
                horizontal_alignment: HorizontalAlign::Center,
            }
            .into(),
            Text {
                text: "bb",
                color: Color::BLACK,
                font_size: 12.0,
                font: "Bravura",
                xy: XY { x: 7.0, y: 8.0 },
                vertical_alignment: VerticalAlign::Top,
                horizontal_alignment: HorizontalAlign::Left,
            }
            .into(),
        ];

        let flat = to_flat_buffer(elements);

        assert_eq!(
            flat.geometry,
            vec![
                TAG_TEXT, 5.0, 6.0, 12.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, // "a": center/middle
                TAG_TEXT, 7.0, 8.0, 12.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, // "bb": left/top
            ]
        );
        assert_eq!(
            flat.text_blob,
            format!("a{}bb", lib::drawable::flat_buffer::TEXT_DELIMITER)
        );
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

        let flat = to_flat_buffer(elements);

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
