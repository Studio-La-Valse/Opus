#[cfg(test)]
mod tests {
    use lib::drawable::drawable_element::{DrawableElement, scale_elem};
    use lib::drawable::elements::line::Line;
    use lib::drawable::elements::polygon::Polygon;
    use lib::drawable::elements::rect::Rect;
    use lib::drawable::elements::text::{HorizontalAlign, Text, VerticalAlign};
    use lib::geometry::color::Color;
    use lib::geometry::xy::XY;

    #[test]
    fn line_scale_scales_endpoints_and_stroke_width_but_not_color() {
        let line = Line {
            start: XY { x: 1.0, y: 2.0 },
            end: XY { x: 3.0, y: 4.0 },
            stroke_color: Color::RED,
            stroke_width: 2.0,
        };

        let scaled = line.scale(0.5);

        assert_eq!((scaled.start.x, scaled.start.y), (0.5, 1.0));
        assert_eq!((scaled.end.x, scaled.end.y), (1.5, 2.0));
        assert_eq!(scaled.stroke_width, 1.0);
        assert_eq!(
            (
                scaled.stroke_color.r,
                scaled.stroke_color.g,
                scaled.stroke_color.b,
                scaled.stroke_color.a
            ),
            (255, 0, 0, 1.0)
        );
    }

    #[test]
    fn rect_scale_scales_geometry_and_optional_stroke_width() {
        let rect = Rect {
            xy: XY { x: 2.0, y: 4.0 },
            width: 10.0,
            height: 20.0,
            color: Color::WHITE,
            stroke_color: Some(Color::BLACK),
            stroke_width: Some(2.0),
        };

        let scaled = rect.scale(0.5);

        assert_eq!((scaled.xy.x, scaled.xy.y), (1.0, 2.0));
        assert_eq!(scaled.width, 5.0);
        assert_eq!(scaled.height, 10.0);
        assert_eq!(scaled.stroke_width, Some(1.0));
    }

    #[test]
    fn rect_scale_leaves_absent_stroke_width_absent() {
        let rect = Rect {
            xy: XY::ZERO,
            width: 10.0,
            height: 20.0,
            color: Color::WHITE,
            stroke_color: None,
            stroke_width: None,
        };

        let scaled = rect.scale(0.5);

        assert_eq!(scaled.stroke_width, None);
    }

    #[test]
    fn text_scale_scales_position_and_font_size_but_not_text_content() {
        let text = Text {
            text: "a",
            color: Color::BLACK,
            font_size: 12.0,
            font: "Bravura",
            xy: XY { x: 4.0, y: 8.0 },
            vertical_alignment: VerticalAlign::Middle,
            horizontal_alignment: HorizontalAlign::Center,
        };

        let scaled = text.scale(0.25);

        assert_eq!((scaled.xy.x, scaled.xy.y), (1.0, 2.0));
        assert_eq!(scaled.font_size, 3.0);
        assert_eq!(scaled.text, "a");
    }

    #[test]
    fn polygon_scale_scales_every_point_and_optional_stroke_width() {
        let polygon = Polygon {
            pts: vec![XY { x: 2.0, y: 4.0 }, XY { x: 6.0, y: 8.0 }],
            color: Color::GREEN,
            stroke_color: None,
            stroke_width: Some(4.0),
        };

        let scaled = polygon.scale(0.5);

        assert_eq!(
            scaled.pts.iter().map(|p| (p.x, p.y)).collect::<Vec<_>>(),
            vec![(1.0, 2.0), (3.0, 4.0)]
        );
        assert_eq!(scaled.stroke_width, Some(2.0));
    }

    #[test]
    fn scale_elem_dispatches_to_the_matching_variant_for_every_shape() {
        let elements: Vec<DrawableElement<'_>> = vec![
            Line {
                start: XY { x: 2.0, y: 2.0 },
                end: XY { x: 4.0, y: 4.0 },
                stroke_color: Color::RED,
                stroke_width: 2.0,
            }
            .into(),
            Rect {
                xy: XY { x: 2.0, y: 2.0 },
                width: 4.0,
                height: 4.0,
                color: Color::WHITE,
                stroke_color: None,
                stroke_width: None,
            }
            .into(),
            Text {
                text: "a",
                color: Color::BLACK,
                font_size: 4.0,
                font: "Bravura",
                xy: XY { x: 2.0, y: 2.0 },
                vertical_alignment: VerticalAlign::Top,
                horizontal_alignment: HorizontalAlign::Left,
            }
            .into(),
            Polygon {
                pts: vec![XY { x: 2.0, y: 2.0 }],
                color: Color::GREEN,
                stroke_color: None,
                stroke_width: None,
            }
            .into(),
        ];

        let scaled: Vec<DrawableElement<'_>> =
            elements.iter().map(|el| scale_elem(el, 0.5)).collect();

        for el in &scaled {
            match el {
                DrawableElement::Line(l) => assert_eq!((l.start.x, l.start.y), (1.0, 1.0)),
                DrawableElement::Rect(r) => assert_eq!((r.xy.x, r.xy.y), (1.0, 1.0)),
                DrawableElement::Text(t) => assert_eq!((t.xy.x, t.xy.y), (1.0, 1.0)),
                DrawableElement::Polygon(p) => assert_eq!((p.pts[0].x, p.pts[0].y), (1.0, 1.0)),
            }
        }
    }
}
