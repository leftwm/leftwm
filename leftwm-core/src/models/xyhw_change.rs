use crate::models::Window;
use crate::models::Xyhw;
use serde::{Deserialize, Serialize};

use super::Handle;

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub struct XyhwChange {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub h: Option<i32>,
    pub w: Option<i32>,
    pub minw: Option<i32>,
    pub maxw: Option<i32>,
    pub minh: Option<i32>,
    pub maxh: Option<i32>,
}

impl From<Xyhw> for XyhwChange {
    fn from(xywh: Xyhw) -> Self {
        Self {
            x: Some(xywh.x()),
            y: Some(xywh.y()),
            w: Some(xywh.w()),
            h: Some(xywh.h()),
            minw: Some(xywh.minw()),
            maxw: Some(xywh.maxw()),
            minh: Some(xywh.minh()),
            maxh: Some(xywh.maxh()),
        }
    }
}

impl XyhwChange {
    pub fn update(&self, xyhw: &mut Xyhw) -> bool {
        let mut changed = false;
        if let Some(x) = self.x {
            if xyhw.x() != x {
                xyhw.set_x(x);
                changed = true;
            }
        }
        if let Some(y) = self.y {
            if xyhw.y() != y {
                xyhw.set_y(y);
                changed = true;
            }
        }
        if let Some(w) = self.w {
            if xyhw.w() != w {
                xyhw.set_w(w);
                changed = true;
            }
        }
        if let Some(h) = self.h {
            if xyhw.h() != h {
                xyhw.set_h(h);
                changed = true;
            }
        }
        if let Some(minw) = self.minw {
            if xyhw.minw() != minw {
                xyhw.set_minw(minw);
                changed = true;
            }
        }
        if let Some(maxw) = self.maxw {
            if xyhw.maxw() != maxw {
                xyhw.set_maxw(maxw);
                changed = true;
            }
        }
        if let Some(minh) = self.minh {
            if xyhw.minh() != minh {
                xyhw.set_minh(minh);
                changed = true;
            }
        }
        if let Some(maxh) = self.maxh {
            if xyhw.maxh() != maxh {
                xyhw.set_maxh(maxh);
                changed = true;
            }
        }
        changed
    }

    pub fn update_window_floating<H: Handle>(&self, window: &mut Window<H>) -> bool {
        let mut changed = false;
        if window.floating() {
            let mut current = window.calculated_xyhw();
            changed = self.update(&mut current);
            window.set_floating_exact(current);
        }
        changed
    }

    /// # Panics
    ///
    /// Souldn't panic; `window.strut` is modified to be a `Some` by
    /// the time of `unwrap()`
    pub fn update_window_strut<H: Handle>(&self, window: &mut Window<H>) -> bool {
        let mut changed = if window.strut.is_none() {
            window.strut = Some(Xyhw::default());
            true
        } else {
            false
        };
        let Some(mut xyhw) = window.strut else {
            return false;
        };
        changed = self.update(&mut xyhw) || changed;
        window.strut = Some(xyhw);
        changed
    }
}
