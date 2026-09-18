use serde::{Deserialize, Serialize};

/// Which corner of a window an interactive resize is anchored to.
///
/// The anchored corner follows the pointer and the opposite one stays where it
/// is, so dragging the top left corner grows the window up and to the left
/// rather than dragging its bottom right corner around.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResizeCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    #[default]
    BottomRight,
}

impl ResizeCorner {
    /// The corner of a rectangle closest to the given point.
    ///
    /// The rectangle is split into quadrants around its centre, so a point
    /// anywhere in the upper left quarter resolves to [`Self::TopLeft`].
    #[must_use]
    pub const fn nearest(
        x: i32,
        y: i32,
        rect_x: i32,
        rect_y: i32,
        width: i32,
        height: i32,
    ) -> Self {
        let left = x < rect_x + width / 2;
        let top = y < rect_y + height / 2;
        match (top, left) {
            (true, true) => Self::TopLeft,
            (true, false) => Self::TopRight,
            (false, true) => Self::BottomLeft,
            (false, false) => Self::BottomRight,
        }
    }

    /// Where this corner sits on a rectangle, which is where the pointer is
    /// repositioned when a resize starts.
    #[must_use]
    pub const fn point(self, rect_x: i32, rect_y: i32, width: i32, height: i32) -> (i32, i32) {
        match self {
            Self::TopLeft => (rect_x, rect_y),
            Self::TopRight => (rect_x + width, rect_y),
            Self::BottomLeft => (rect_x, rect_y + height),
            Self::BottomRight => (rect_x + width, rect_y + height),
        }
    }

    /// Whether the left edge moves with the pointer instead of the right one.
    #[must_use]
    pub const fn anchors_left(self) -> bool {
        matches!(self, Self::TopLeft | Self::BottomLeft)
    }

    /// Whether the top edge moves with the pointer instead of the bottom one.
    #[must_use]
    pub const fn anchors_top(self) -> bool {
        matches!(self, Self::TopLeft | Self::TopRight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_resolves_each_quadrant() {
        // A 100x100 window at the origin, one point per quadrant.
        assert_eq!(
            ResizeCorner::nearest(10, 10, 0, 0, 100, 100),
            ResizeCorner::TopLeft
        );
        assert_eq!(
            ResizeCorner::nearest(90, 10, 0, 0, 100, 100),
            ResizeCorner::TopRight
        );
        assert_eq!(
            ResizeCorner::nearest(10, 90, 0, 0, 100, 100),
            ResizeCorner::BottomLeft
        );
        assert_eq!(
            ResizeCorner::nearest(90, 90, 0, 0, 100, 100),
            ResizeCorner::BottomRight
        );
    }

    #[test]
    fn nearest_accounts_for_the_window_origin() {
        // The same offsets within a window that does not start at the origin.
        assert_eq!(
            ResizeCorner::nearest(510, 210, 500, 200, 100, 100),
            ResizeCorner::TopLeft
        );
        assert_eq!(
            ResizeCorner::nearest(590, 290, 500, 200, 100, 100),
            ResizeCorner::BottomRight
        );
    }

    #[test]
    fn the_centre_resolves_to_the_bottom_right() {
        // The centre belongs to no quadrant, and falling back to the corner
        // LeftWM has always used keeps the old behaviour for that one pixel.
        assert_eq!(
            ResizeCorner::nearest(50, 50, 0, 0, 100, 100),
            ResizeCorner::BottomRight
        );
    }

    #[test]
    fn point_returns_the_matching_corner() {
        assert_eq!(ResizeCorner::TopLeft.point(500, 200, 100, 50), (500, 200));
        assert_eq!(ResizeCorner::TopRight.point(500, 200, 100, 50), (600, 200));
        assert_eq!(
            ResizeCorner::BottomLeft.point(500, 200, 100, 50),
            (500, 250)
        );
        assert_eq!(
            ResizeCorner::BottomRight.point(500, 200, 100, 50),
            (600, 250)
        );
    }

    #[test]
    fn anchors_report_the_moving_edges() {
        assert!(ResizeCorner::TopLeft.anchors_left() && ResizeCorner::TopLeft.anchors_top());
        assert!(!ResizeCorner::TopRight.anchors_left() && ResizeCorner::TopRight.anchors_top());
        assert!(ResizeCorner::BottomLeft.anchors_left() && !ResizeCorner::BottomLeft.anchors_top());
        assert!(
            !ResizeCorner::BottomRight.anchors_left() && !ResizeCorner::BottomRight.anchors_top()
        );
    }

    #[test]
    fn the_default_is_the_historical_corner() {
        assert_eq!(ResizeCorner::default(), ResizeCorner::BottomRight);
    }
}
