use crate::models::Window;
use crate::models::XYHWBuilder;
use crate::models::XYHW;
use log::warn;

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct XYHWChange {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub h: Option<i32>,
    pub w: Option<i32>,
    pub minw: Option<i32>,
    pub maxw: Option<i32>,
    pub minh: Option<i32>,
    pub maxh: Option<i32>,
}

impl Default for XYHWChange {
    fn default() -> Self {
        XYHWChange {
            x: None,
            y: None,
            w: None,
            h: None,
            minw: None,
            maxw: None,
            minh: None,
            maxh: None,
        }
    }
}

impl From<XYHW> for XYHWChange {
    fn from(xywh: XYHW) -> Self {
        XYHWChange {
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

impl XYHWChange {
    pub fn update(&self, xyhw: &mut XYHW) -> bool {
        let mut changed = false;
        if let Some(x) = self.x {
            if xyhw.x() != x {
                warn!("CHANGE: X");
                xyhw.set_x(x);
                changed = true;
            }
        }
        if let Some(y) = self.y {
            if xyhw.y() != y {
                warn!("CHANGE: Y");
                xyhw.set_y(y);
                changed = true;
            }
        }
        if let Some(w) = self.w {
            if xyhw.w() != w {
                warn!("CHANGE: W");
                xyhw.set_w(w);
                changed = true;
            }
        }
        if let Some(h) = self.h {
            if xyhw.h() != h {
                warn!("CHANGE: H");
                xyhw.set_h(h);
                changed = true;
            }
        }
        if let Some(minw) = self.minw {
            if xyhw.minw() != minw {
                warn!("CHANGE: minw");
                xyhw.set_minw(minw);
                changed = true;
            }
        }
        if let Some(maxw) = self.maxw {
            if xyhw.maxw() != maxw {
                warn!("CHANGE: maxw");
                xyhw.set_maxw(maxw);
                changed = true;
            }
        }
        if let Some(minh) = self.minh {
            if xyhw.minh() != minh {
                warn!("CHANGE: minh");
                xyhw.set_minh(minh);
                changed = true;
            }
        }
        if let Some(maxh) = self.maxh {
            if xyhw.maxh() != maxh {
                warn!("CHANGE: maxh");
                xyhw.set_maxh(maxh);
                changed = true;
            }
        }
        changed
    }

    pub fn update_window(&self, window: &mut Window) -> bool {
        let mut changed = false;
        if window.floating() {
            let mut current = window.calculated_xyhw();
            changed = self.update(&mut current);
            window.set_floating_exact(current);
        }
        changed
    }
}
