use super::layouts::*;
use crate::models::BBox;
use crate::models::Tag;
use crate::models::Window;
use crate::models::WindowType;
use crate::models::XYHWBuilder;
use crate::models::XYHW;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone)]
pub struct Workspace {
    pub id: i32,
    /// Active layout
    pub layout: Layout,
    pub tags: Vec<Tag>,
    pub avoid: Vec<XYHW>,
    pub xyhw: XYHW,
    xyhw_avoided: XYHW,
}

impl fmt::Debug for Workspace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Workspace {{ id: {}, tags: {:?}, x: {}, y: {} }}",
            self.id,
            self.tags,
            self.xyhw.x(),
            self.xyhw.y()
        )
    }
}

impl PartialEq for Workspace {
    fn eq(&self, other: &Workspace) -> bool {
        self.id != -1 && self.id == other.id
    }
}

impl Workspace {
    pub fn new(bbox: BBox) -> Workspace {
        Workspace {
            id: -1,
            layout: Layout::default(),
            tags: vec![],
            avoid: vec![],

            xyhw: XYHWBuilder {
                h: bbox.height,
                w: bbox.width,
                x: bbox.x,
                y: bbox.y,
                ..Default::default()
            }
            .into(),
            xyhw_avoided: XYHWBuilder {
                h: bbox.height,
                w: bbox.width,
                x: bbox.x,
                y: bbox.y,
                ..Default::default()
            }
            .into(),
        }
    }

    pub fn show_tag(&mut self, tag: Tag) {
        self.tags = vec![tag];
    }

    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        self.xyhw.contains_point(x, y)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for t in &self.tags {
            if tag == t {
                return true;
            }
        }
        false
    }

    pub fn next_layout(&mut self) {
        self.layout = self.layout.next_layout();
    }

    pub fn prev_layout(&mut self) {
        self.layout = self.layout.prev_layout();
    }

    /// Returns true if the workspace is displays a given window.
    pub fn is_displaying(&self, window: &Window) -> bool {
        for wd_t in &window.tags {
            if self.has_tag(wd_t) {
                return true;
            }
        }
        false
    }

    /// Returns true if the workspace is to update the locations info of this window.
    pub fn is_managed(&self, window: &Window) -> bool {
        self.is_displaying(window) && window.type_ != WindowType::Dock
    }

    pub fn update_windows(&self, windows: &mut Vec<&mut Window>) {
        //mark all windows for this workspace as visible
        let mut all_mine: Vec<&mut &mut Window> = windows
            .iter_mut()
            .filter(|w| self.is_displaying(w))
            .collect();
        all_mine.iter_mut().for_each(|w| w.set_visible(true));
        //update the location of all non-floating windows
        let mut managed_nonfloat: Vec<&mut &mut Window> = windows
            .iter_mut()
            .filter(|w| self.is_managed(w) && !w.floating())
            .collect();
        self.layout.update_windows(self, &mut managed_nonfloat);
        //update the location of all floating windows
        windows
            .iter_mut()
            .filter(|w| self.is_managed(w) && w.floating())
            .for_each(|w| w.normal = self.xyhw);
    }

    pub fn x(&self) -> i32 {
        self.xyhw_avoided.x()
    }
    pub fn y(&self) -> i32 {
        self.xyhw_avoided.y()
    }
    pub fn height(&self) -> i32 {
        self.xyhw_avoided.h()
    }
    pub fn width(&self) -> i32 {
        self.xyhw_avoided.w()
    }

    pub fn center_halfed(&self) -> XYHW {
        self.xyhw_avoided.center_halfed()
    }

    pub fn update_avoided_areas(&mut self) {
        let mut xyhw = self.xyhw;
        for a in &self.avoid {
            xyhw = xyhw.without(a);
        }
        self.xyhw_avoided = xyhw;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BBox, WindowHandle};

    #[test]
    fn empty_ws_should_not_contain_window() {
        let subject = Workspace::new(BBox {
            width: 600,
            height: 800,
            x: 0,
            y: 0,
        });
        let w = Window::new(WindowHandle::MockHandle(1), None);
        assert!(
            !subject.is_displaying(&w),
            "workspace incorrectly owns window"
        );
    }

    #[test]
    fn tagging_a_workspace_to_with_the_same_tag_as_a_window_should_couse_it_to_display() {
        let mut subject = Workspace::new(BBox {
            width: 600,
            height: 800,
            x: 0,
            y: 0,
        });
        subject.show_tag("test".to_owned());
        let mut w = Window::new(WindowHandle::MockHandle(1), None);
        w.tag("test".to_owned());
        assert!(subject.is_displaying(&w), "workspace should include window");
    }
}
