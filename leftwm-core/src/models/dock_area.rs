use crate::models::Xyhw;
use crate::models::XyhwBuilder;

use super::Handle;
use super::Screen;

#[derive(Copy, Clone, Debug, Default)]
pub struct DockArea {
    pub top: i32,
    pub top_start_x: i32,
    pub top_end_x: i32,

    pub bottom: i32,
    pub bottom_start_x: i32,
    pub bottom_end_x: i32,

    pub right: i32,
    pub right_start_y: i32,
    pub right_end_y: i32,

    pub left: i32,
    pub left_start_y: i32,
    pub left_end_y: i32,
}

// impl From<&[i64]> for DockArea {
//     fn from(slice: &[i64]) -> Self {
//         Self {
//             left: slice[0] as i32,
//             right: slice[1] as i32,
//             top: slice[2] as i32,
//             bottom: slice[3] as i32,
//             left_start_y: slice[4] as i32,
//             left_end_y: slice[5] as i32,
//             right_start_y: slice[6] as i32,
//             right_end_y: slice[7] as i32,
//             top_start_x: slice[8] as i32,
//             top_end_x: slice[9] as i32,
//             bottom_start_x: slice[10] as i32,
//             bottom_end_x: slice[11] as i32,
//         }
//     }
// }
//
// impl From<&[i32]> for DockArea {
//     fn from(slice: &[i32]) -> Self {
//         Self {
//             left: slice[0],
//             right: slice[1],
//             top: slice[2],
//             bottom: slice[3],
//             left_start_y: slice[4],
//             left_end_y: slice[5],
//             right_start_y: slice[6],
//             right_end_y: slice[7],
//             top_start_x: slice[8],
//             top_end_x: slice[9],
//             bottom_start_x: slice[10],
//             bottom_end_x: slice[11],
//         }
//     }
// }

impl DockArea {
    #[must_use]
    pub fn as_xyhw<H: Handle>(
        &self,
        screens_height: i32,
        screens_width: i32,
        screen: &Screen<H>,
    ) -> Option<Xyhw> {
        if self.top > 0 {
            return Some(self.xyhw_from_top(screen.bbox.y));
        }
        if self.bottom > 0 {
            return Some(self.xyhw_from_bottom(screens_height, screen.bbox.y + screen.bbox.height));
        }
        if self.left > 0 {
            return Some(self.xyhw_from_left(screen.bbox.x));
        }
        if self.right > 0 {
            return Some(self.xyhw_from_right(screens_width, screen.bbox.x + screen.bbox.width));
        }
        None
    }

    fn xyhw_from_top(&self, screen_y: i32) -> Xyhw {
        XyhwBuilder {
            x: self.top_start_x,
            y: screen_y,
            h: self.top - screen_y,
            w: self.top_end_x - self.top_start_x,
            ..XyhwBuilder::default()
        }
        .into()
    }

    fn xyhw_from_bottom(&self, screens_height: i32, screen_bottom: i32) -> Xyhw {
        XyhwBuilder {
            x: self.bottom_start_x,
            y: screens_height - self.bottom,
            h: self.bottom - (screens_height - screen_bottom),
            w: self.bottom_end_x - self.bottom_start_x,
            ..XyhwBuilder::default()
        }
        .into()
    }

    fn xyhw_from_left(&self, screen_x: i32) -> Xyhw {
        XyhwBuilder {
            x: screen_x,
            y: self.left_start_y,
            h: self.left_end_y - self.left_start_y,
            w: self.left - screen_x,
            ..XyhwBuilder::default()
        }
        .into()
    }

    fn xyhw_from_right(&self, screens_width: i32, screen_right: i32) -> Xyhw {
        XyhwBuilder {
            x: screens_width - self.right,
            y: self.right_start_y,
            h: self.right_end_y - self.right_start_y,
            w: self.right - (screens_width - screen_right),
            ..XyhwBuilder::default()
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_able_to_build_from_top() {
        let area = DockArea {
            top: 2,
            top_start_x: 10,
            top_end_x: 200,
            ..DockArea::default()
        };
        let expected: Xyhw = XyhwBuilder {
            h: 2,
            w: 190,
            x: 10,
            y: 0,
            ..XyhwBuilder::default()
        }
        .into();
        assert_eq!(area.xyhw_from_top(0), expected);
    }

    #[test]
    fn should_be_able_to_build_from_bottom() {
        let area = DockArea {
            bottom: 2,
            bottom_start_x: 10,
            bottom_end_x: 200,
            ..DockArea::default()
        };
        let expected: Xyhw = XyhwBuilder {
            h: 2,
            w: 190,
            x: 10,
            y: 998,
            ..XyhwBuilder::default()
        }
        .into();
        assert_eq!(area.xyhw_from_bottom(1000, 1000), expected);
    }

    #[test]
    fn should_be_able_to_build_from_left() {
        let area = DockArea {
            left: 2,
            left_start_y: 10,
            left_end_y: 200,
            ..DockArea::default()
        };
        let expected: Xyhw = XyhwBuilder {
            h: 190,
            w: 2,
            x: 0,
            y: 10,
            ..XyhwBuilder::default()
        }
        .into();
        assert_eq!(area.xyhw_from_left(0), expected);
    }

    #[test]
    fn should_be_able_to_build_from_right() {
        let area = DockArea {
            right: 2,
            right_start_y: 10,
            right_end_y: 200,
            ..DockArea::default()
        };
        let expected: Xyhw = XyhwBuilder {
            h: 190,
            w: 2,
            x: 1998,
            y: 10,
            ..XyhwBuilder::default()
        }
        .into();
        assert_eq!(area.xyhw_from_right(2000, 2000), expected);
    }
}
