#[cfg(test)]
mod tests {
    use lib::geometry::ray::Ray;
    use lib::geometry::xy::XY;

    fn xy(x: f32, y: f32) -> XY {
        XY { x, y }
    }

    /// Assert two points are equal within a small tolerance.
    fn assert_close(a: XY, b: XY) {
        let tol = 1e-3;
        assert!(
            (a.x - b.x).abs() <= tol && (a.y - b.y).abs() <= tol,
            "points differ: ({}, {}) vs ({}, {})",
            a.x,
            a.y,
            b.x,
            b.y
        );
    }

    #[test]
    fn from_dir_keeps_origin_and_dir() {
        let r = Ray::from_dir(xy(1.0, 2.0), xy(3.0, 4.0));
        assert_eq!(r.origin.x, 1.0);
        assert_eq!(r.origin.y, 2.0);
        assert_eq!(r.dir.x, 3.0);
        assert_eq!(r.dir.y, 4.0);
    }

    #[test]
    fn from_pts_builds_dir_as_difference() {
        let r = Ray::from_pts(xy(1.0, 1.0), xy(4.0, 5.0));
        assert_eq!(r.origin.x, 1.0);
        assert_eq!(r.origin.y, 1.0);
        assert_eq!(r.dir.x, 3.0);
        assert_eq!(r.dir.y, 4.0);
    }

    #[test]
    fn mv_shifts_origin_only() {
        let r = Ray::from_dir(xy(1.0, 2.0), xy(3.0, 4.0)).mv(10.0, -5.0);
        assert_eq!(r.origin.x, 11.0);
        assert_eq!(r.origin.y, -3.0);
        assert_eq!(r.dir.x, 3.0);
        assert_eq!(r.dir.y, 4.0);
    }

    #[test]
    fn valid_is_false_for_zero_dir() {
        assert!(!Ray::from_dir(xy(0.0, 0.0), xy(0.0, 0.0)).valid());
        assert!(Ray::from_dir(xy(0.0, 0.0), xy(1.0, 0.0)).valid());
    }

    // --- intersect: the order-independence question ---

    #[test]
    fn intersect_is_order_independent_for_clean_crossing() {
        // Horizontal ray along y = 2, vertical ray along x = 3. They cross at (3, 2).
        let horizontal = Ray::from_dir(xy(0.0, 2.0), xy(1.0, 0.0));
        let vertical = Ray::from_dir(xy(3.0, 0.0), xy(0.0, 1.0));

        let a = horizontal.intersect(vertical).unwrap();
        let b = vertical.intersect(horizontal).unwrap();

        assert_eq!(a.x, 3.0);
        assert_eq!(a.y, 2.0);
        // For these axis-aligned integer inputs the result is bit-identical both ways.
        assert_eq!(a.x, b.x);
        assert_eq!(a.y, b.y);
    }

    #[test]
    fn intersect_is_order_independent_for_skewed_rays() {
        // Two diagonally skewed rays with awkward coordinates. The geometric
        // intersection point is unique, so both call orders must agree — though
        // only up to floating-point rounding, since each order parameterises the
        // solution along a different direction vector.
        let left = Ray::from_pts(xy(-7.3, 1.1), xy(2.6, 9.4));
        let right = Ray::from_pts(xy(0.0, 11.7), xy(13.2, -4.8));

        let a = left.intersect(right).unwrap();
        let b = right.intersect(left).unwrap();

        assert_close(a, b);
    }

    #[test]
    fn intersect_point_lies_on_both_rays() {
        let left = Ray::from_pts(xy(-7.3, 1.1), xy(2.6, 9.4));
        let right = Ray::from_pts(xy(0.0, 11.7), xy(13.2, -4.8));

        let p = left.intersect(right).unwrap();

        // p must satisfy the implicit line equation of each ray:
        // cross(dir, p - origin) == 0
        let on = |r: &Ray, p: XY| r.dir.x * (p.y - r.origin.y) - r.dir.y * (p.x - r.origin.x);
        assert!(on(&left, p).abs() <= 1e-2, "off left ray: {}", on(&left, p));
        assert!(
            on(&right, p).abs() <= 1e-2,
            "off right ray: {}",
            on(&right, p)
        );
    }

    #[test]
    fn intersect_is_independent_of_dir_magnitude() {
        let a = Ray::from_dir(xy(0.0, 2.0), xy(1.0, 0.0));
        let b = Ray::from_dir(xy(3.0, 0.0), xy(0.0, 1.0));
        let b_long = Ray::from_dir(xy(3.0, 0.0), xy(0.0, 1000.0));

        let p = a.intersect(b).unwrap();
        let p_long = a.intersect(b_long).unwrap();

        assert_close(p, p_long);
    }

    #[test]
    fn parallel_rays_return_none_in_both_orders() {
        let a = Ray::from_dir(xy(0.0, 0.0), xy(1.0, 1.0));
        let b = Ray::from_dir(xy(1.0, 0.0), xy(1.0, 1.0));

        assert!(a.intersect(b).is_none());
        assert!(b.intersect(a).is_none());
    }

    #[test]
    fn collinear_rays_return_none_in_both_orders() {
        let a = Ray::from_dir(xy(0.0, 0.0), xy(2.0, 1.0));
        let b = Ray::from_dir(xy(4.0, 2.0), xy(-2.0, -1.0));

        assert!(a.intersect(b).is_none());
        assert!(b.intersect(a).is_none());
    }

    #[test]
    fn degenerate_ray_returns_none_in_both_orders() {
        let real = Ray::from_dir(xy(0.0, 0.0), xy(1.0, 0.0));
        let degenerate = Ray::from_dir(xy(2.0, 5.0), xy(0.0, 0.0));

        assert!(real.intersect(degenerate).is_none());
        assert!(degenerate.intersect(real).is_none());
    }

    #[test]
    fn intersect_when_one_origin_is_the_crossing_point() {
        // right starts exactly on the horizontal ray at (5, 0).
        let horizontal = Ray::from_dir(xy(0.0, 0.0), xy(1.0, 0.0));
        let right = Ray::from_dir(xy(5.0, 0.0), xy(1.0, 3.0));

        let a = horizontal.intersect(right).unwrap();
        let b = right.intersect(horizontal).unwrap();

        assert_close(a, xy(5.0, 0.0));
        assert_close(a, b);
    }
}
