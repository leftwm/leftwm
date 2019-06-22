use std::cmp;
use std::ops::Add;
use std::ops::Sub;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Copy)]
pub struct XYHW {
    x: i32,
    y: i32,
    h: i32,
    w: i32,
    minw: i32,
    maxw: i32,
    minh: i32,
    maxh: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Copy)]
pub struct XYHWBuilder {
    pub x: i32,
    pub y: i32,
    pub h: i32,
    pub w: i32,
    pub minw: i32,
    pub maxw: i32,
    pub minh: i32,
    pub maxh: i32,
}

impl Default for XYHWBuilder {
    fn default() -> Self {
        XYHWBuilder {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            minw: 5,
            maxw: 999_999_999,
            minh: 5,
            maxh: 999_999_999,
        }
    }
}

impl Default for XYHW {
    fn default() -> Self {
        XYHW {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            minw: 5,
            maxw: 999_999_999,
            minh: 5,
            maxh: 999_999_999,
        }
    }
}

impl Add for XYHW {
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

impl Sub for XYHW {
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

impl From<XYHWBuilder> for XYHW {
    fn from(xywh: XYHWBuilder) -> Self {
        let mut b = XYHW {
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

impl XYHW {
    pub fn x(&self) -> i32 {
        self.x
    }
    pub fn y(&self) -> i32 {
        self.y
    }
    pub fn h(&self) -> i32 {
        self.h
    }
    pub fn w(&self) -> i32 {
        self.w
    }

    pub fn minw(&self) -> i32 {
        self.minw
    }
    pub fn maxw(&self) -> i32 {
        self.maxw
    }
    pub fn minh(&self) -> i32 {
        self.minh
    }
    pub fn maxh(&self) -> i32 {
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
            self.h = self.maxh
        }
        if self.w > self.maxw {
            self.w = self.maxw
        }
        if self.h < self.minh {
            self.h = self.minh
        }
        if self.w < self.minw {
            self.w = self.minw
        }
    }

    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        let maxx = self.x + self.w;
        let maxy = self.y + self.h;
        (self.x <= x && x <= maxx) && (self.y <= y && y <= maxy)
    }

    pub fn volume(&self) -> u64 {
        self.h as u64 * self.w as u64
    }

    /*
     * Trim a XYHW out of another XYHW so that they don't overlap
     */
    pub fn without(&self, other: &XYHW) -> XYHW {
        let mut without = *self;
        if other.w > other.h {
            //horizontal trim
            if other.y > self.y + (self.h / 2) {
                //bottom check
                let bottom_over = (without.y + without.h) - other.y;
                if bottom_over > 0 {
                    without.h -= bottom_over;
                }
            } else {
                //top check
                let top_over = (other.y + other.h) - without.y;
                if top_over > 0 {
                    without.y += top_over;
                    without.h -= top_over;
                }
            }
        } else {
            //vertical trim
            let left_over = (other.x + other.w) - without.x;
            if other.x > self.x + (self.w / 2) {
                //right check
                let right_over = (without.x + without.w) - other.x;
                if right_over > 0 {
                    without.w -= right_over;
                }
            } else {
                //left check
                if left_over > 0 {
                    without.x += left_over;
                    without.w -= left_over;
                }
            }
        }
        without
    }

    pub fn center_halfed(&self) -> XYHW {
        XYHWBuilder {
            x: self.x + (self.w / 2) - (self.w / 4),
            y: self.y + (self.h / 2) - (self.h / 4),
            h: (self.h / 2),
            w: (self.w / 2),
            ..Default::default()
        }
        .into()
    }

    pub fn center(&self) -> (i32, i32) {
        let x = self.x + (self.w / 2);
        let y = self.y + (self.h / 2);
        (x, y)
    }
}

#[test]
fn without_should_trim_from_the_top() {
    let a = XYHW {
        y: 5,
        h: 1000,
        w: 1000,
        ..Default::default()
    };
    let b = XYHW {
        h: 10,
        w: 100,
        ..Default::default()
    };
    let result = a.without(&b);
    assert_eq!(
        result,
        XYHW {
            x: 0,
            y: 10,
            h: 995,
            w: 1000,
            ..Default::default()
        }
    );
}

#[test]
fn without_should_trim_from_the_left() {
    let a = XYHW {
        x: 0,
        y: 0,
        h: 1000,
        w: 1000,
        ..Default::default()
    };
    let b = XYHW {
        h: 100,
        w: 10,
        ..Default::default()
    };
    let result = a.without(&b);
    assert_eq!(
        result,
        XYHW {
            x: 10,
            y: 0,
            w: 990,
            h: 1000,
            ..Default::default()
        }
    );
}

#[test]
fn without_should_trim_from_the_bottom() {
    let a = XYHW {
        x: 0,
        y: 0,
        h: 1000,
        w: 1000,
        ..Default::default()
    };
    let b = XYHW {
        y: 990,
        x: 0,
        h: 10,
        w: 100,
        ..Default::default()
    };
    let result = a.without(&b);
    assert_eq!(
        result,
        XYHW {
            x: 0,
            y: 0,
            h: 990,
            w: 1000,
            ..Default::default()
        }
    );
}

#[test]
fn without_should_trim_from_the_right() {
    let a = XYHW {
        x: 0,
        y: 0,
        h: 1000,
        w: 1000,
        ..Default::default()
    };
    let b = XYHW {
        x: 990,
        y: 0,
        h: 100,
        w: 10,
        ..Default::default()
    };
    let result = a.without(&b);
    assert_eq!(
        result,
        XYHW {
            x: 0,
            y: 0,
            w: 990,
            h: 1000,
            ..Default::default()
        }
    );
}
