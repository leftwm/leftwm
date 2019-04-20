#[derive(Clone, Debug, PartialEq, Copy)]
pub struct XYHW {
    pub x: i32,
    pub y: i32,
    pub h: i32,
    pub w: i32,
}

impl Default for XYHW {
    fn default() -> Self {
        XYHW {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        }
    }
}

impl XYHW {
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
        XYHW {
            x: self.x + (self.w / 2) - (self.w / 4),
            y: self.y + (self.h / 2) - (self.h / 4),
            h: (self.h / 2),
            w: (self.w / 2),
        }
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
    };
    let b = XYHW {
        y: 990,
        x: 0,
        h: 10,
        w: 100,
    };
    let result = a.without(&b);
    assert_eq!(
        result,
        XYHW {
            x: 0,
            y: 0,
            h: 990,
            w: 1000,
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
    };
    let b = XYHW {
        x: 990,
        y: 0,
        h: 100,
        w: 10,
    };
    let result = a.without(&b);
    assert_eq!(
        result,
        XYHW {
            x: 0,
            y: 0,
            w: 990,
            h: 1000,
        }
    );
}
