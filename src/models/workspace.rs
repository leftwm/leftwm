use super::{layouts::Layout, Margins};
use crate::models::Gutter;
use crate::models::Side;
use crate::models::Tag;
use crate::models::TagId;
use crate::models::Window;
use crate::models::Xyhw;
use crate::models::XyhwBuilder;
use crate::{config::ThemeSetting, models::BBox};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Information for workspaces (screen divisions).
#[derive(Serialize, Deserialize, Clone)]
pub struct Workspace {
    pub id: Option<i32>,
    /// Active layout
    pub layout: Layout,
    layout_rotation: usize,
    pub tags: Vec<TagId>,
    pub margin: Margins,
    pub margin_multiplier: f32,
    pub gutters: Vec<Gutter>,
    // We allow dead code here, as >1.56.0 complains
    // This should be investigated further.
    #[allow(dead_code)]
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
            "Workspace {{ id: {:?}, tags: {:?}, x: {}, y: {} }}",
            self.id,
            self.tags,
            self.xyhw.x(),
            self.xyhw.y()
        )
    }
}

impl PartialEq for Workspace {
    fn eq(&self, other: &Self) -> bool {
        self.id != None && self.id == other.id
    }
}

impl Workspace {
    #[must_use]
    pub fn new(id: Option<i32>, bbox: BBox, all_tags: Vec<Tag>, layouts: Vec<Layout>) -> Self {
        Self {
            id,
            layout: Layout::new(&layouts),
            layout_rotation: 0,
            tags: vec![],
            margin: Margins::Int(10),
            margin_multiplier: 1.0,
            gutters: vec![],
            avoid: vec![],
            all_tags,
            layouts,
            xyhw: XyhwBuilder {
                h: bbox.height,
                w: bbox.width,
                x: bbox.x,
                y: bbox.y,
                ..XyhwBuilder::default()
            }
            .into(),
            xyhw_avoided: XyhwBuilder {
                h: bbox.height,
                w: bbox.width,
                x: bbox.x,
                y: bbox.y,
                ..XyhwBuilder::default()
            }
            .into(),
        }
    }

    pub fn update_for_theme(&mut self, theme: &ThemeSetting) {
        self.margin = theme.workspace_margin.clone().unwrap_or(Margins::Int(0));
        self.gutters = self.get_gutters_for_theme(theme);
    }

    pub fn get_gutters_for_theme(&mut self, theme: &ThemeSetting) -> Vec<Gutter> {
        theme
            .get_list_of_gutters()
            .into_iter()
            .filter(|gutter| gutter.wsid == self.id || gutter.wsid == None)
            .fold(vec![], |mut acc, gutter| {
                match acc.iter().enumerate().find(|(_i, g)| g.side == gutter.side) {
                    Some((i, x)) => {
                        if x.wsid.is_none() {
                            acc[i] = gutter;
                        }
                    }
                    None => acc.push(gutter),
                }
                acc
            })
    }

    pub fn show_tag(&mut self, tags: &mut Vec<Tag>, tag: &Tag) {
        self.tags = vec![tag.id.clone()];
        self.set_main_width(tags, self.layout.main_width());
    }

    #[must_use]
    pub const fn contains_point(&self, x: i32, y: i32) -> bool {
        self.xyhw.contains_point(x, y)
    }

    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        for t in &self.tags {
            if tag == t {
                return true;
            }
        }
        false
    }

    pub fn next_layout(&mut self, tags: &mut Vec<Tag>) {
        self.layout = self.layout.next_layout(&self.layouts);
        self.set_main_width(tags, self.layout.main_width());
        self.layout_rotation = 0;
    }

    pub fn prev_layout(&mut self, tags: &mut Vec<Tag>) {
        self.layout = self.layout.prev_layout(&self.layouts);
        self.set_main_width(tags, self.layout.main_width());
        self.layout_rotation = 0;
    }

    pub fn set_layout(&mut self, tags: &mut Vec<Tag>, layout: Layout) {
        self.layout = layout;
        self.set_main_width(tags, self.layout.main_width());
        self.layout_rotation = 0;
    }

    /// Returns true if the workspace is displays a given window.
    #[must_use]
    pub fn is_displaying(&self, window: &Window) -> bool {
        for wd_t in &window.tags {
            if self.has_tag(wd_t) {
                return true;
            }
        }
        false
    }

    /// Returns true if the workspace is to update the locations info of this window.
    #[must_use]
    pub fn is_managed(&self, window: &Window) -> bool {
        self.is_displaying(window) && !window.is_unmanaged()
    }

    pub fn update_windows(&self, windows: &mut Vec<Window>, tags: &mut Vec<Tag>) {
        if let Some(w) = windows
            .iter_mut()
            .find(|w| self.is_displaying(w) && w.is_fullscreen())
        {
            w.set_visible(true);
        } else {
            //Don't bother updating the other windows
            //mark all windows for this workspace as visible
            let mut all_mine: Vec<&mut Window> = windows
                .iter_mut()
                .filter(|w| self.is_displaying(w))
                .collect();
            all_mine.iter_mut().for_each(|w| w.set_visible(true));
            //update the location of all non-floating windows
            let mut managed_nonfloat: Vec<&mut Window> = windows
                .iter_mut()
                .filter(|w| self.is_managed(w) && !w.floating())
                .collect();
            self.layout
                .update_windows(self, &mut managed_nonfloat, tags);
            for w in &mut managed_nonfloat {
                w.container_size = Some(self.xyhw);
            }
            //update the location of all floating windows
            windows
                .iter_mut()
                .filter(|w| self.is_managed(w) && w.floating())
                .for_each(|w| w.normal = self.xyhw);
        }
    }

    #[must_use]
    pub fn x(&self) -> i32 {
        let left = self.margin.clone().left() as f32;
        let gutter = self.get_gutter(&Side::Left);
        self.xyhw_avoided.x() + (self.margin_multiplier * left) as i32 + gutter
    }
    #[must_use]
    pub fn y(&self) -> i32 {
        let top = self.margin.clone().top() as f32;
        let gutter = self.get_gutter(&Side::Top);
        self.xyhw_avoided.y() + (self.margin_multiplier * top) as i32 + gutter
    }
    #[must_use]
    pub fn height(&self) -> i32 {
        let top = self.margin.clone().top() as f32;
        let bottom = self.margin.clone().bottom() as f32;
        //Only one side
        let gutter = self.get_gutter(&Side::Top) + self.get_gutter(&Side::Bottom);
        self.xyhw_avoided.h() - (self.margin_multiplier * (top + bottom)) as i32 - gutter
    }
    #[must_use]
    pub fn width(&self) -> i32 {
        let left = self.margin.clone().left() as f32;
        let right = self.margin.clone().right() as f32;
        //Only one side
        let gutter = self.get_gutter(&Side::Left) + self.get_gutter(&Side::Right);
        self.xyhw_avoided.w() - (self.margin_multiplier * (left + right)) as i32 - gutter
    }

    fn get_gutter(&self, side: &Side) -> i32 {
        match self.gutters.iter().find(|g| &g.side == side) {
            Some(g) => g.value,
            None => 0,
        }
    }

    #[must_use]
    pub fn current_tags<'a>(&self, tags: &'a mut Vec<Tag>) -> Vec<&'a mut Tag> {
        tags.iter_mut()
            .filter(|tag| self.tags.contains(&tag.id))
            .collect()
    }

    pub fn change_main_width(&self, tags: &mut Vec<Tag>, delta: i8) {
        self.current_tags(tags)
            .iter_mut()
            .for_each(|t| t.change_main_width(delta));
    }

    pub fn set_main_width(&self, tags: &mut Vec<Tag>, val: u8) {
        if let Some(tag) = self.current_tags(tags).get_mut(0) {
            tag.set_main_width(val);
        }
    }

    #[must_use]
    pub fn main_width(&self, tags: &mut Vec<Tag>) -> f32 {
        if let Some(tag) = self.current_tags(tags).get(0) {
            return tag.main_width_percentage();
        }
        f32::from(self.layout.main_width())
    }

    pub fn rotate_layout(&mut self, tags: &mut Vec<Tag>) -> Option<()> {
        let rotations = self.layout.rotations();
        self.layout_rotation += 1;
        if self.layout_rotation >= rotations.len() {
            self.layout_rotation = 0;
        }
        let (horz, vert) = rotations.get(self.layout_rotation)?;
        let tags = &mut self.current_tags(tags);
        let tag = tags.get_mut(0)?;
        tag.flipped_horizontal = *horz;
        tag.flipped_vertical = *vert;
        Some(())
    }

    #[must_use]
    pub fn flipped_horizontal(&self, tags: &mut Vec<Tag>) -> bool {
        if let Some(tag) = self.current_tags(tags).get(0) {
            return tag.flipped_horizontal;
        }
        false
    }

    #[must_use]
    pub fn flipped_vertical(&self, tags: &mut Vec<Tag>) -> bool {
        if let Some(tag) = self.current_tags(tags).get(0) {
            return tag.flipped_vertical;
        }
        false
    }

    #[must_use]
    pub fn center_halfed(&self) -> Xyhw {
        self.xyhw_avoided.center_halfed()
    }

    pub fn right_bottom(&self) -> Xyhw {
        self.xyhw_avoided.right_bottom_corner()
    }

    pub fn update_avoided_areas(&mut self) {
        let mut xyhw = self.xyhw;
        for a in &self.avoid {
            xyhw = xyhw.without(a);
        }
        self.xyhw_avoided = xyhw;
    }

    /// Set the tag model's margin multiplier.
    pub fn set_margin_multiplier(&mut self, margin_multiplier: f32) {
        self.margin_multiplier = margin_multiplier;
    }

    /// Get a reference to the tag model's margin multiplier.
    #[must_use]
    pub const fn margin_multiplier(&self) -> f32 {
        self.margin_multiplier
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BBox, WindowHandle};

    #[test]
    fn empty_ws_should_not_contain_window() {
        let subject = Workspace::new(
            None,
            BBox {
                width: 600,
                height: 800,
                x: 0,
                y: 0,
            },
            vec![],
            vec![],
        );
        let w = Window::new(WindowHandle::MockHandle(1), None, None);
        assert!(
            !subject.is_displaying(&w),
            "workspace incorrectly owns window"
        );
    }

    #[test]
    fn tagging_a_workspace_to_with_the_same_tag_as_a_window_should_couse_it_to_display() {
        let mut subject = Workspace::new(
            None,
            BBox {
                width: 600,
                height: 800,
                x: 0,
                y: 0,
            },
            vec![],
            vec![],
        );
        let tag = crate::models::Tag::new("test");
        let mut tags = vec![tag.clone()];
        subject.show_tag(&mut tags, &tag);
        let mut w = Window::new(WindowHandle::MockHandle(1), None, None);
        w.tag("test");
        assert!(subject.is_displaying(&w), "workspace should include window");
    }
}
