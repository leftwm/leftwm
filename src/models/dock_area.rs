use crate::models::Xyhw;
use crate::models::XyhwBuilder;

#[derive(Clone, Debug, Default)]
pub struct DockArea {
    top: i32,
    top_start_x: i32,
    top_end_x: i32,

    bottom: i32,
    bottom_start_x: i32,
    bottom_end_x: i32,

    right: i32,
    right_start_y: i32,
    right_end_y: i32,

    left: i32,
    left_start_y: i32,
    left_end_y: i32,
}

impl From<&[i64]> for DockArea {
    fn from(slice: &[i64]) -> Self {
        DockArea {
            left: slice[0] as i32,
            right: slice[1] as i32,
            top: slice[2] as i32,
            bottom: slice[3] as i32,
            left_start_y: slice[4] as i32,
            left_end_y: slice[5] as i32,
            right_start_y: slice[6] as i32,
            right_end_y: slice[7] as i32,
            top_start_x: slice[8] as i32,
            top_end_x: slice[9] as i32,
            bottom_start_x: slice[10] as i32,
            bottom_end_x: slice[11] as i32,
        }
    }
}

impl DockArea {
    pub fn as_xyhw(&self, screen_height: i32, screen_width: i32) -> Option<Xyhw> {
        if self.top > 0 {
            return Some(self.xyhw_from_top());
        }
        if self.bottom > 0 {
            return Some(self.xyhw_from_bottom(screen_height));
        }
        if self.left > 0 {
            return Some(self.xyhw_from_left());
        }
        if self.right > 0 {
            return Some(self.xyhw_from_right(screen_width));
        }
        None
    }

    fn xyhw_from_top(&self) -> Xyhw {
        XyhwBuilder {
            x: self.top_start_x,
            y: 0,
            h: self.top,
            w: self.top_end_x - self.top_start_x,
            ..Default::default()
        }
        .into()
    }

    fn xyhw_from_bottom(&self, screen_height: i32) -> Xyhw {
        XyhwBuilder {
            x: self.bottom_start_x,
            y: screen_height - self.bottom,
            h: self.bottom,
            w: self.bottom_end_x - self.bottom_start_x,
            ..Default::default()
        }
        .into()
    }

    fn xyhw_from_left(&self) -> Xyhw {
        XyhwBuilder {
            x: 0,
            y: self.left_start_y,
            h: self.left_end_y - self.left_start_y,
            w: self.left,
            ..Default::default()
        }
        .into()
    }

    fn xyhw_from_right(&self, screen_width: i32) -> Xyhw {
        XyhwBuilder {
            x: screen_width - self.right,
            y: self.right_start_y,
            h: self.right_end_y - self.right_start_y,
            w: self.right,
            ..Default::default()
        }
        .into()
    }
}

#[cfg(tests)]
mod tests {
    use super::*;

    #[test]
    fn should_be_able_to_build_from_top() {
        let area = DockArea {
            top: 2,
            top_start_x: 10,
            top_end_x: 200,
            ..Default::default()
        };
        let expected: Xyhw = XyhwBuilder {
            h: 2,
            w: 190,
            x: 10,
            y: 0,
            ..Default::default()
        }
        .into();
        assert_eq!(area.xyhw_from_top(), expected);
    }

    #[test]
    fn should_be_able_to_build_from_bottom() {
        let area = DockArea {
            bottom: 2,
            bottom_start_x: 10,
            bottom_end_x: 200,
            ..Default::default()
        };
        let expected: Xyhw = XyhwBuilder {
            h: 2,
            w: 190,
            x: 10,
            y: 998,
            ..Default::default()
        }
        .into();
        assert_eq!(area.xyhw_from_bottom(1000), expected);
    }

    #[test]
    fn should_be_able_to_build_from_left() {
        let area = DockArea {
            left: 2,
            left_start_y: 10,
            left_end_y: 200,
            ..Default::default()
        };
        let expected: Xyhw = XyhwBuilder {
            h: 190,
            w: 2,
            x: 0,
            y: 10,
            ..Default::default()
        }
        .into();
        assert_eq!(area.xyhw_from_left(), expected);
    }

    #[test]
    fn should_be_able_to_build_from_right() {
        let area = DockArea {
            right: 2,
            right_start_y: 10,
            right_end_y: 200,
            ..Default::default()
        };
        let expected: Xyhw = XyhwBuilder {
            h: 190,
            w: 2,
            x: 1998,
            y: 10,
            ..Default::default()
        }
        .into();
        assert_eq!(area.xyhw_from_right(2000), expected);
    }
}
