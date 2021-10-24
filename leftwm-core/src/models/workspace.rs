use crate::config::Config;
use crate::models::{
    layouts::Layout, BBox, Gutter, Margins, Side, Size, Tag, TagId, Window, Xyhw, XyhwBuilder,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Information for workspaces (screen divisions).
#[derive(Serialize, Deserialize, Clone)]
pub struct Workspace {
    pub id: Option<i32>,
    /// Active layout
    pub layout: Layout,
    pub main_width_percentage: u8,
    pub tags: Vec<TagId>,
    pub margin: Margins,
    pub margin_multiplier: f32,
    pub gutters: Vec<Gutter>,
    #[serde(skip)]
    pub avoid: Vec<Xyhw>,
    pub xyhw: Xyhw,
    xyhw_avoided: Xyhw,
    pub max_window_width: Option<Size>,
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
    pub fn new(
        id: Option<i32>,
        bbox: BBox,
        layout: Layout,
        max_window_width: Option<Size>,
    ) -> Self {
        Self {
            id,
            layout,
            main_width_percentage: layout.main_width(),
            tags: vec![],
            margin: Margins::new(10),
            margin_multiplier: 1.0,
            gutters: vec![],
            avoid: vec![],
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
            max_window_width,
        }
    }

    pub fn update_for_theme(&mut self, config: &impl Config) {
        self.margin = config.workspace_margin().unwrap_or_else(|| Margins::new(0));
        self.gutters = self.get_gutters_for_theme(config);
    }

    pub fn get_gutters_for_theme(&mut self, config: &impl Config) -> Vec<Gutter> {
        config
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

    pub fn show_tag(&mut self, tag: &Tag) {
        self.tags = vec![tag.id.clone()];
    }

    #[must_use]
    pub const fn contains_point(&self, x: i32, y: i32) -> bool {
        self.xyhw.contains_point(x, y)
    }

    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_owned())
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

    /// Returns the original x position of the workspace,
    /// disregarding the optional `max_window_width` configuration
    #[must_use]
    pub fn x(&self) -> i32 {
        let left = self.margin.left as f32;
        let gutter = self.get_gutter(&Side::Left);
        self.xyhw_avoided.x() + (self.margin_multiplier * left) as i32 + gutter
    }

    /// Returns the x position for the workspace,
    /// while accounting for the optional `max_window_width` configuration
    #[must_use]
    pub fn x_limited(&self, column_count: usize) -> i32 {
        match self.width() - self.width_limited(column_count) {
            0 => self.x(),
            remainder => self.x() + (remainder / 2),
        }
    }

    #[must_use]
    pub fn y(&self) -> i32 {
        let top = self.margin.top as f32;
        let gutter = self.get_gutter(&Side::Top);
        self.xyhw_avoided.y() + (self.margin_multiplier * top) as i32 + gutter
    }

    #[must_use]
    pub fn height(&self) -> i32 {
        let top = self.margin.top as f32;
        let bottom = self.margin.bottom as f32;
        //Only one side
        let gutter = self.get_gutter(&Side::Top) + self.get_gutter(&Side::Bottom);
        self.xyhw_avoided.h() - (self.margin_multiplier * (top + bottom)) as i32 - gutter
    }

    /// Returns the original width for the workspace,
    /// disregarding the optional `max_window_width` configuration
    #[must_use]
    pub fn width(&self) -> i32 {
        let left = self.margin.left as f32;
        let right = self.margin.right as f32;
        //Only one side
        let gutter = self.get_gutter(&Side::Left) + self.get_gutter(&Side::Right);
        self.xyhw_avoided.w() - (self.margin_multiplier * (left + right)) as i32 - gutter
    }

    /// Returns the width of the workspace,
    /// while accounting for the optional `max_window_width` configuration
    #[must_use]
    pub fn width_limited(&self, column_count: usize) -> i32 {
        match self.max_window_width {
            Some(size) => std::cmp::min(
                (size.into_absolute(self.width() as f32) * column_count as f32).floor() as i32,
                self.width(),
            ),
            None => self.width(),
        }
    }

    fn get_gutter(&self, side: &Side) -> i32 {
        match self.gutters.iter().find(|g| &g.side == side) {
            Some(g) => g.value,
            None => 0,
        }
    }

    #[must_use]
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

    /// Set the tag model's margin multiplier.
    pub fn set_margin_multiplier(&mut self, margin_multiplier: f32) {
        self.margin_multiplier = margin_multiplier;
    }

    /// Get a reference to the tag model's margin multiplier.
    #[must_use]
    pub const fn margin_multiplier(&self) -> f32 {
        self.margin_multiplier
    }

    pub fn change_main_width(&mut self, delta: i8) {
        //Check we are not gonna go negative
        let mwp = &mut self.main_width_percentage;
        if (*mwp as i8) < -delta {
            *mwp = 0;
            return;
        }
        if delta.is_negative() {
            *mwp -= delta.unsigned_abs();
            return;
        }
        *mwp += delta as u8;
        if *mwp > 100 {
            *mwp = 100;
        }
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
            Layout::default(),
            None,
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
            Layout::default(),
            None,
        );
        let tag = crate::models::Tag::new("test", Layout::default());
        subject.show_tag(&tag);
        let mut w = Window::new(WindowHandle::MockHandle(1), None, None);
        w.tag("test");
        assert!(subject.is_displaying(&w), "workspace should include window");
    }
}
