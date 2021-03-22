use super::layouts::*;
use crate::models::BBox;
use crate::models::Tag;
use crate::models::TagId;
use crate::models::Window;
use crate::models::WindowType;
use crate::models::Xyhw;
use crate::models::XyhwBuilder;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone)]
pub struct Workspace {
    pub id: i32,
    /// Active layout
    pub layout: Layout,
    pub tags: Vec<TagId>,
    #[serde(skip)]
    all_tags: Vec<Tag>,
    layouts: Vec<Layout>,
    pub avoid: Vec<Xyhw>,
    pub xyhw: Xyhw,
    xyhw_avoided: Xyhw,
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
    pub fn new(bbox: BBox, all_tags: Vec<Tag>, layouts: Vec<Layout>) -> Workspace {
        Workspace {
            id: -1,
            layout: Layout::new(&layouts),
            tags: vec![],
            avoid: vec![],
            all_tags,
            layouts,
            xyhw: XyhwBuilder {
                h: bbox.height,
                w: bbox.width,
                x: bbox.x,
                y: bbox.y,
                ..Default::default()
            }
            .into(),
            xyhw_avoided: XyhwBuilder {
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
        self.tags = vec![tag.id.clone()];
        self.set_main_width(self.layout.main_width());
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
        self.layout = self.layout.next_layout(&self.layouts);
        self.set_main_width(self.layout.main_width());
    }

    pub fn prev_layout(&mut self) {
        self.layout = self.layout.prev_layout(&self.layouts);
        self.set_main_width(self.layout.main_width());
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
        managed_nonfloat
            .iter_mut()
            .for_each(|w| w.container_size = Some(self.xyhw));
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

    pub fn current_tags(&self) -> Vec<Tag> {
        let mut found = vec![];
        for tag in &self.all_tags {
            if self.tags.contains(&tag.id) {
                found.push(tag.clone());
            }
        }
        found
    }

    pub fn increase_main_width(&self, delta: u8) {
        for tag in self.current_tags() {
            tag.increase_main_width(delta);
        }
    }
    pub fn decrease_main_width(&self, delta: u8) {
        for tag in self.current_tags() {
            tag.decrease_main_width(delta);
        }
    }

    pub fn set_main_width(&self, val: u8) {
        if let Some(tag) = self.current_tags().get(0) {
            tag.set_main_width(val);
        }
    }

    pub fn main_width(&self) -> f32 {
        if let Some(tag) = self.current_tags().get(0) {
            return tag.main_width_percentage();
        }
        self.layout.main_width() as f32
    }

    pub fn center_halfed(&self) -> Xyhw {
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
        let subject = Workspace::new(
            BBox {
                width: 600,
                height: 800,
                x: 0,
                y: 0,
            },
            vec![],
            vec![],
        );
        let w = Window::new(WindowHandle::MockHandle(1), None);
        assert!(
            !subject.is_displaying(&w),
            "workspace incorrectly owns window"
        );
    }

    #[test]
    fn tagging_a_workspace_to_with_the_same_tag_as_a_window_should_couse_it_to_display() {
        let mut subject = Workspace::new(
            BBox {
                width: 600,
                height: 800,
                x: 0,
                y: 0,
            },
            vec![],
            vec![],
        );
        let tag = crate::models::TagModel::new("test");
        subject.show_tag(tag);
        let mut w = Window::new(WindowHandle::MockHandle(1), None);
        w.tag("test");
        assert!(subject.is_displaying(&w), "workspace should include window");
    }
}
