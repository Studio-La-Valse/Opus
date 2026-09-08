#[cfg(test)]
mod tests {
    use lib::drawable::canvas::CanvasPainter;
    use lib::drawable::canvas::flat_buffer::{FlatBufferCanvas, TAG_RECT, TAG_TEXT};
    use lib::drawable::canvas::svg::SvgCanvas;
    use lib::drawable::drawable_element::{DrawableElement, compute_bounds};
    use lib::drawable::elements::text::{FontSpec, HorizontalAlign, Text, VerticalAlign};
    use lib::geometry::bounding_box::BoundingBox;
    use lib::geometry::color::Color;
    use lib::geometry::xy::XY;

    /// A 40 x 20 box with its corner at (10, 100): edges at x 10 and 50, y 100
    /// and 120, middles at x 30 and y 110.
    fn a_box() -> BoundingBox {
        BoundingBox {
            xy: XY { x: 10.0, y: 100.0 },
            size: XY { x: 40.0, y: 20.0 },
        }
    }

    fn text(bounds: BoundingBox, h: HorizontalAlign, v: VerticalAlign) -> Text<'static> {
        Text {
            text: "lyric",
            color: Color::BLACK,
            font_size: 10.0,
            font: FontSpec::plain("serif"),
            bounds,
            vertical_alignment: v,
            horizontal_alignment: h,
            background: None,
        }
    }

    fn on_white(mut t: Text<'static>) -> Text<'static> {
        t.background = Some(Color::WHITE);
        t
    }

    /// The three vertical alignments read off the box's three horizontal lines:
    /// hanging from the top, centred on the middle, sitting on the bottom as a
    /// baseline.
    #[test]
    fn vertical_alignment_picks_one_of_the_boxs_three_lines() {
        for (align, expected) in [
            (VerticalAlign::Top, 100.0),
            (VerticalAlign::Middle, 110.0),
            (VerticalAlign::Bottom, 120.0),
        ] {
            let anchor = text(a_box(), HorizontalAlign::Left, align).anchor();
            assert_eq!(anchor.y, expected, "{align:?}");
        }
    }

    #[test]
    fn horizontal_alignment_picks_one_of_the_boxs_three_verticals() {
        for (align, expected) in [
            (HorizontalAlign::Left, 10.0),
            (HorizontalAlign::Center, 30.0),
            (HorizontalAlign::Right, 50.0),
        ] {
            let anchor = text(a_box(), align, VerticalAlign::Top).anchor();
            assert_eq!(anchor.x, expected, "{align:?}");
        }
    }

    /// A box with no size has all three of its lines in the same place, so it
    /// anchors at its corner whatever the alignments -- which is how a producer
    /// with nothing to reserve still positions text by a bare point.
    #[test]
    fn a_zero_sized_box_anchors_at_its_corner_under_every_alignment() {
        let at = XY { x: 7.0, y: 9.0 };

        for h in [
            HorizontalAlign::Left,
            HorizontalAlign::Center,
            HorizontalAlign::Right,
        ] {
            for v in [
                VerticalAlign::Top,
                VerticalAlign::Middle,
                VerticalAlign::Bottom,
            ] {
                let anchor = text(BoundingBox::point(at), h, v).anchor();
                assert_eq!((anchor.x, anchor.y), (7.0, 9.0), "{h:?} / {v:?}");
            }
        }
    }

    /// The box is what the element occupies, so it is what it contributes to
    /// the render bounds -- not the anchor the alignments happen to pick out of
    /// it, which for a centred text is nowhere near an edge.
    #[test]
    fn a_text_contributes_its_whole_box_to_the_render_bounds() {
        let elements: Vec<DrawableElement<'_>> =
            vec![text(a_box(), HorizontalAlign::Center, VerticalAlign::Middle).into()];

        assert_eq!(compute_bounds(&elements), (10.0, 100.0, 50.0, 120.0));
    }

    /// The sink is handed the anchor and the alignment mode that matches it:
    /// `end` / `baseline` against the box's bottom-right corner puts the run
    /// inside the box, ending there.
    #[test]
    fn svg_draws_from_the_anchor_the_box_resolves_to() {
        let elements: Vec<DrawableElement<'_>> =
            vec![text(a_box(), HorizontalAlign::Right, VerticalAlign::Bottom).into()];
        let svg = CanvasPainter::new(SvgCanvas::new((0.0, 0.0, 200.0, 200.0))).paint(&elements);

        assert!(
            svg.contains(r#"<text x="50" y="120""#),
            "expected the box's bottom-right corner: {svg}"
        );
        assert!(
            svg.contains(r#"text-anchor="end" dominant-baseline="baseline""#),
            "{svg}"
        );
    }

    /// The background covers the whole box, not the run inside it, and goes
    /// down before the text so the text sits on top of it.
    #[test]
    fn svg_fills_the_background_over_the_box_behind_the_text() {
        let elements: Vec<DrawableElement<'_>> =
            vec![on_white(text(a_box(), HorizontalAlign::Left, VerticalAlign::Top)).into()];
        let svg = CanvasPainter::new(SvgCanvas::new((0.0, 0.0, 200.0, 200.0))).paint(&elements);

        let rect =
            r##"<rect x="10" y="100" width="40" height="20" fill="#FFFFFFFF" stroke="none""##;
        assert!(svg.contains(rect), "expected {rect} in {svg}");
        assert!(
            svg.find(rect) < svg.find("<text"),
            "background painted over the text: {svg}"
        );
    }

    /// No background is the absence of a rect, not a transparent one -- a text
    /// that wants none costs nothing to draw.
    #[test]
    fn svg_emits_nothing_extra_without_a_background() {
        let elements: Vec<DrawableElement<'_>> =
            vec![text(a_box(), HorizontalAlign::Left, VerticalAlign::Top).into()];
        let svg = CanvasPainter::new(SvgCanvas::new((0.0, 0.0, 200.0, 200.0))).paint(&elements);

        assert!(!svg.contains("<rect"), "{svg}");
    }

    /// The buffer needs no background fields and the decoder no new case: a
    /// background is a rect record ahead of the text record, so one element
    /// contributes two records.
    #[test]
    fn the_flat_buffer_emits_a_background_as_a_rect_record() {
        let elements: Vec<DrawableElement<'_>> =
            vec![on_white(text(a_box(), HorizontalAlign::Left, VerticalAlign::Top)).into()];
        let flat = CanvasPainter::new(FlatBufferCanvas::new()).paint(&elements);

        assert_eq!(
            flat.geometry,
            vec![
                // the box, filled white, unstroked ...
                TAG_RECT, 10.0, 100.0, 40.0, 20.0, 255.0, 255.0, 255.0, 1.0, -1.0, 0.0, 0.0, 0.0,
                0.0, //
                // ... then the run, anchored at its top-left
                TAG_TEXT, 10.0, 100.0, 10.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
            ]
        );
    }

    /// The flat buffer is a drawing format, so it carries the resolved anchor
    /// rather than the box -- a sink needs the anchor and the alignment modes,
    /// and nothing else, to place the run.
    #[test]
    fn the_flat_buffer_record_carries_the_resolved_anchor() {
        let elements: Vec<DrawableElement<'_>> =
            vec![text(a_box(), HorizontalAlign::Center, VerticalAlign::Middle).into()];
        let flat = CanvasPainter::new(FlatBufferCanvas::new()).paint(&elements);

        assert_eq!(
            flat.geometry,
            vec![
                // the box's two middle lines, then center/middle, then the font
                TAG_TEXT, 30.0, 110.0, 10.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0,
            ]
        );
    }
}
