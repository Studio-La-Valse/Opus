#[cfg(test)]
mod tests {
    use lib::bounding_box::BoundingBox;
    use lib::xy::XY;

    #[test]
    fn width_and_height() {
        let bb = BoundingBox {
            xy: XY { x: 0.0, y: 0.0 },
            size: XY { x: 10.0, y: 20.0 },
        };

        assert_eq!(bb.width(), 10.0);
        assert_eq!(bb.height(), 20.0);
    }

    #[test]
    fn min_max_coordinates() {
        let bb = BoundingBox {
            xy: XY { x: 5.0, y: 7.0 },
            size: XY { x: 3.0, y: 4.0 },
        };

        assert_eq!(bb.x_min(), 5.0);
        assert_eq!(bb.x_max(), 8.0);
        assert_eq!(bb.y_min(), 7.0);
        assert_eq!(bb.y_max(), 11.0);
    }

    #[test]
    fn intersects_true() {
        let a = BoundingBox {
            xy: XY { x: 0.0, y: 0.0 },
            size: XY { x: 10.0, y: 10.0 },
        };
        let b = BoundingBox {
            xy: XY { x: 5.0, y: 5.0 },
            size: XY { x: 10.0, y: 10.0 },
        };

        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn intersects_false() {
        let a = BoundingBox {
            xy: XY { x: 0.0, y: 0.0 },
            size: XY { x: 10.0, y: 10.0 },
        };
        let b = BoundingBox {
            xy: XY { x: 11.0, y: 11.0 },
            size: XY { x: 5.0, y: 5.0 },
        };

        assert!(!a.intersects(&b));
        assert!(!b.intersects(&a));
    }

    #[test]
    fn intersects_touching_edges_is_false() {
        let a = BoundingBox {
            xy: XY { x: 0.0, y: 0.0 },
            size: XY { x: 10.0, y: 10.0 },
        };
        let b = BoundingBox {
            xy: XY { x: 10.0, y: 0.0 },
            size: XY { x: 5.0, y: 5.0 },
        };

        assert!(!a.intersects(&b));
    }

    #[test]
    fn intersection_some() {
        let a = BoundingBox {
            xy: XY { x: 0.0, y: 0.0 },
            size: XY { x: 10.0, y: 10.0 },
        };
        let b = BoundingBox {
            xy: XY { x: 5.0, y: 3.0 },
            size: XY { x: 10.0, y: 10.0 },
        };

        let inter = a.intersection(&b).unwrap();

        assert_eq!(inter.xy.x, 5.0);
        assert_eq!(inter.xy.y, 3.0);
        assert_eq!(inter.size.x, 5.0);
        assert_eq!(inter.size.y, 7.0);
    }

    #[test]
    fn intersection_none() {
        let a = BoundingBox {
            xy: XY { x: 0.0, y: 0.0 },
            size: XY { x: 10.0, y: 10.0 },
        };
        let b = BoundingBox {
            xy: XY { x: 20.0, y: 20.0 },
            size: XY { x: 5.0, y: 5.0 },
        };

        assert!(a.intersection(&b).is_none());
    }

    #[test]
    fn contains_box_true() {
        let outer = BoundingBox {
            xy: XY { x: 0.0, y: 0.0 },
            size: XY { x: 10.0, y: 10.0 },
        };
        let inner = BoundingBox {
            xy: XY { x: 2.0, y: 2.0 },
            size: XY { x: 3.0, y: 3.0 },
        };

        assert!(outer.contains_box(&inner));
    }

    #[test]
    fn contains_box_false() {
        let outer = BoundingBox {
            xy: XY { x: 0.0, y: 0.0 },
            size: XY { x: 10.0, y: 10.0 },
        };
        let inner = BoundingBox {
            xy: XY { x: 9.0, y: 9.0 },
            size: XY { x: 3.0, y: 3.0 },
        };

        assert!(!outer.contains_box(&inner));
    }

    #[test]
    fn contains_box_edge_epsilon() {
        let outer = BoundingBox {
            xy: XY { x: 0.0, y: 0.0 },
            size: XY { x: 10.0, y: 10.0 },
        };
        let touching = BoundingBox {
            xy: XY { x: 10.0, y: 10.0 },
            size: XY { x: 0.0, y: 0.0 },
        };

        assert!(outer.contains_box(&touching));
    }

    #[test]
    fn zero_box() {
        let bb = BoundingBox::ZERO;
        assert!(bb.is_zero());
    }

    #[test]
    fn scale_box() {
        let bb = BoundingBox {
            xy: XY { x: 1.0, y: 2.0 },
            size: XY { x: 3.0, y: 4.0 },
        };

        let scaled = bb.scale(2.0);

        assert_eq!(scaled.xy.x, 1.0);
        assert_eq!(scaled.xy.y, 2.0);
        assert_eq!(scaled.size.x, 6.0);
        assert_eq!(scaled.size.y, 8.0);
    }

    #[test]
    fn move_box() {
        let bb = BoundingBox {
            xy: XY { x: 1.0, y: 2.0 },
            size: XY { x: 3.0, y: 4.0 },
        };

        let moved = bb.mv(5.0, -2.0);

        assert_eq!(moved.xy.x, 6.0);
        assert_eq!(moved.xy.y, 0.0);
        assert_eq!(moved.size.x, 3.0);
        assert_eq!(moved.size.y, 4.0);
    }
}
