//! Various window and workspace sizing structs.
#![allow(clippy::module_name_repetitions)]
use leftwm_layouts::geometry::Rect;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::ops::Add;
use std::ops::Sub;

/// Struct containing min/max width and height and window placement. x,y from top left.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub struct Xyhw {
    x: i32,
    y: i32,
    h: i32,
    w: i32,
    minw: i32,
    maxw: i32,
    minh: i32,
    maxh: i32,
}

/// Modifiable struct that can be used to generate an Xyhw struct. Contains min/max width and
/// height and window placement. x,y from top left.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub struct XyhwBuilder {
    pub x: i32,
    pub y: i32,
    pub h: i32,
    pub w: i32,
    pub minw: i32,
    pub maxw: i32,
    pub minh: i32,
    pub maxh: i32,
}

impl Default for XyhwBuilder {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            minw: -999_999_999,
            maxw: 999_999_999,
            minh: -999_999_999,
            maxh: 999_999_999,
        }
    }
}

impl Default for Xyhw {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            minw: -999_999_999,
            maxw: 999_999_999,
            minh: -999_999_999,
            maxh: 999_999_999,
        }
    }
}

impl Add for Xyhw {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            w: self.w + other.w,
            h: self.h + other.h,
            minw: cmp::max(self.minw, other.minw),
            maxw: cmp::min(self.maxw, other.maxw),
            minh: cmp::max(self.minh, other.minh),
            maxh: cmp::min(self.maxh, other.maxh),
        }
    }
}

impl Sub for Xyhw {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            w: self.w - other.w,
            h: self.h - other.h,
            minw: cmp::max(self.minw, other.minw),
            maxw: cmp::min(self.maxw, other.maxw),
            minh: cmp::max(self.minh, other.minh),
            maxh: cmp::min(self.maxh, other.maxh),
        }
    }
}

impl From<XyhwBuilder> for Xyhw {
    fn from(xywh: XyhwBuilder) -> Self {
        let mut b = Self {
            x: xywh.x,
            y: xywh.y,
            w: xywh.w,
            h: xywh.h,
            minw: xywh.minw,
            maxw: xywh.maxw,
            minh: xywh.minh,
            maxh: xywh.maxh,
        };
        b.update_limits();
        b
    }
}

impl From<Rect> for Xyhw {
    fn from(rect: Rect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            w: rect.w as i32,
            h: rect.h as i32,
            ..Default::default()
        }
    }
}

impl From<Xyhw> for Rect {
    fn from(xyhw: Xyhw) -> Self {
        Rect {
            x: xyhw.x,
            y: xyhw.y,
            w: xyhw.w.unsigned_abs(),
            h: xyhw.h.unsigned_abs(),
        }
    }
}

impl Xyhw {
    #[must_use]
    pub const fn x(&self) -> i32 {
        self.x
    }
    #[must_use]
    pub const fn y(&self) -> i32 {
        self.y
    }
    #[must_use]
    pub const fn h(&self) -> i32 {
        self.h
    }
    #[must_use]
    pub const fn w(&self) -> i32 {
        self.w
    }

    #[must_use]
    pub const fn minw(&self) -> i32 {
        self.minw
    }
    #[must_use]
    pub const fn maxw(&self) -> i32 {
        self.maxw
    }
    #[must_use]
    pub const fn minh(&self) -> i32 {
        self.minh
    }
    #[must_use]
    pub const fn maxh(&self) -> i32 {
        self.maxh
    }

    pub fn clear_minmax(&mut self) {
        self.minw = -999_999_999;
        self.maxw = 999_999_999;
        self.minh = -999_999_999;
        self.maxh = 999_999_999;
        self.update_limits();
    }

    pub fn set_x(&mut self, value: i32) {
        self.x = value;
        self.update_limits();
    }
    pub fn set_y(&mut self, value: i32) {
        self.y = value;
        self.update_limits();
    }
    pub fn set_h(&mut self, value: i32) {
        self.h = value;
        self.update_limits();
    }
    pub fn set_w(&mut self, value: i32) {
        self.w = value;
        self.update_limits();
    }

    pub fn set_minw(&mut self, value: i32) {
        self.minw = value;
        self.update_limits();
    }
    pub fn set_maxw(&mut self, value: i32) {
        self.maxw = value;
        self.update_limits();
    }
    pub fn set_minh(&mut self, value: i32) {
        self.minh = value;
        self.update_limits();
    }
    pub fn set_maxh(&mut self, value: i32) {
        self.maxh = value;
        self.update_limits();
    }

    fn update_limits(&mut self) {
        if self.h > self.maxh {
            self.h = self.maxh;
        }
        if self.w > self.maxw {
            self.w = self.maxw;
        }
        if self.h < self.minh {
            self.h = self.minh;
        }
        if self.w < self.minw {
            self.w = self.minw;
        }
    }

    #[must_use]
    pub const fn contains_point(&self, x: i32, y: i32) -> bool {
        let max_x = self.x + self.w;
        let max_y = self.y + self.h;
        (self.x <= x && x <= max_x) && (self.y <= y && y <= max_y)
    }

    pub const fn contains_xyhw(&self, other: &Self) -> bool {
        let other_max_x = other.x + other.w;
        let other_max_y = other.y + other.h;
        self.contains_point(other.x, other.y) && self.contains_point(other_max_x, other_max_y)
    }

    #[must_use]
    pub const fn volume(&self) -> u64 {
        self.h as u64 * self.w as u64
    }

    /// Trim a Xyhw out of another Xyhw so that they don't overlap.
    #[must_use]
    pub const fn without(&self, other: &Self) -> Self {
        let mut without = *self;
        if other.w > other.h {
            // horizontal trim
            if other.y > self.y + (self.h / 2) {
                // bottom check
                let bottom_over = (without.y + without.h) - other.y;
                if bottom_over > 0 {
                    without.h -= bottom_over;
                }
            } else {
                // top check
                let top_over = (other.y + other.h) - without.y;
                if top_over > 0 {
                    without.y += top_over;
                    without.h -= top_over;
                }
            }
        } else {
            // vertical trim
            let left_over = (other.x + other.w) - without.x;
            if other.x > self.x + (self.w / 2) {
                // right check
                let right_over = (without.x + without.w) - other.x;
                if right_over > 0 {
                    without.w -= right_over;
                }
            } else {
                // left check
                if left_over > 0 {
                    without.x += left_over;
                    without.w -= left_over;
                }
            }
        }
        without
    }

    #[must_use]
    pub fn center_halfed(&self) -> Self {
        XyhwBuilder {
            x: self.x + (self.w / 2) - (self.w / 4),
            y: self.y + (self.h / 2) - (self.h / 4),
            h: (self.h / 2),
            w: (self.w / 2),
            ..XyhwBuilder::default()
        }
        .into()
    }

    pub fn center_relative(&mut self, outer: Self, border: i32) {
        // If this is a Splash screen, for some reason w/h are sometimes not sized properly. We
        // therefore want to use the minimum or maximum sizing instead in order to center the
        // windows properly.
        self.x = outer.x() + outer.w() / 2 - self.w / 2 - border;
        self.y = outer.y() + outer.h() / 2 - self.h / 2 - border;
    }

    pub const fn center(&self) -> (i32, i32) {
        let x = self.x + (self.w / 2);
        let y = self.y + (self.h / 2);
        (x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_halfed() {
        let a = Xyhw {
            x: 10,
            y: 10,
            w: 2000,
            h: 1000,
            ..Xyhw::default()
        };
        let correct = Xyhw {
            x: 510,
            y: 260,
            w: 1000,
            h: 500,
            ..Xyhw::default()
        };
        let result = a.center_halfed();
        assert_eq!(result, correct);
    }

    #[test]
    fn without_should_trim_from_the_top() {
        let a = Xyhw {
            y: 5,
            h: 1000,
            w: 1000,
            ..Xyhw::default()
        };
        let b = Xyhw {
            h: 10,
            w: 100,
            ..Xyhw::default()
        };
        let result = a.without(&b);
        assert_eq!(
            result,
            Xyhw {
                x: 0,
                y: 10,
                h: 995,
                w: 1000,
                ..Xyhw::default()
            }
        );
    }

    #[test]
    fn without_should_trim_from_the_left() {
        let a = Xyhw {
            x: 0,
            y: 0,
            h: 1000,
            w: 1000,
            ..Xyhw::default()
        };
        let b = Xyhw {
            h: 100,
            w: 10,
            ..Xyhw::default()
        };
        let result = a.without(&b);
        assert_eq!(
            result,
            Xyhw {
                x: 10,
                y: 0,
                w: 990,
                h: 1000,
                ..Xyhw::default()
            }
        );
    }

    #[test]
    fn without_should_trim_from_the_bottom() {
        let a = Xyhw {
            x: 0,
            y: 0,
            h: 1000,
            w: 1000,
            ..Xyhw::default()
        };
        let b = Xyhw {
            y: 990,
            x: 0,
            h: 10,
            w: 100,
            ..Xyhw::default()
        };
        let result = a.without(&b);
        assert_eq!(
            result,
            Xyhw {
                x: 0,
                y: 0,
                h: 990,
                w: 1000,
                ..Xyhw::default()
            }
        );
    }

    #[test]
    fn without_should_trim_from_the_right() {
        let a = Xyhw {
            x: 0,
            y: 0,
            h: 1000,
            w: 1000,
            ..Xyhw::default()
        };
        let b = Xyhw {
            x: 990,
            y: 0,
            h: 100,
            w: 10,
            ..Xyhw::default()
        };
        let result = a.without(&b);
        assert_eq!(
            result,
            Xyhw {
                x: 0,
                y: 0,
                w: 990,
                h: 1000,
                ..Xyhw::default()
            }
        );
    }

    #[test]
    fn contains_xyhw_should_detect_a_inner_window() {
        let a = Xyhw {
            x: 0,
            y: 0,
            h: 1000,
            w: 1000,
            ..Xyhw::default()
        };
        let b = Xyhw {
            x: 100,
            y: 100,
            h: 800,
            w: 800,
            ..Xyhw::default()
        };
        assert!(a.contains_xyhw(&b));
    }

    #[test]
    fn contains_xyhw_should_detect_a_upper_left_corner_outside() {
        let a = Xyhw {
            x: 100,
            y: 100,
            h: 800,
            w: 800,
            ..Xyhw::default()
        };
        let b = Xyhw {
            x: 0,
            y: 0,
            h: 200,
            w: 200,
            ..Xyhw::default()
        };
        assert!(!a.contains_xyhw(&b));
    }

    #[test]
    fn contains_xyhw_should_detect_a_lower_right_corner_outside() {
        let a = Xyhw {
            x: 100,
            y: 100,
            h: 800,
            w: 800,
            ..Xyhw::default()
        };
        let b = Xyhw {
            x: 800,
            y: 800,
            h: 200,
            w: 200,
            ..Xyhw::default()
        };
        assert!(!a.contains_xyhw(&b));
    }
}
