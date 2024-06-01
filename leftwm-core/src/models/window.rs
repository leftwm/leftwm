//! Window Information
#![allow(clippy::module_name_repetitions)]

use std::fmt::Debug;

use super::WindowState;
use super::WindowType;
use crate::models::Margins;
use crate::models::TagId;
use crate::models::Xyhw;
use crate::models::XyhwBuilder;
use crate::Workspace;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// A trait which backend specific window handles need to implement
pub trait Handle:
    Serialize + DeserializeOwned + Debug + Clone + Copy + PartialEq + Eq + Default + Send + 'static
{
}

/// A Backend-agnostic handle to a window used to identify it
///
/// # Serde
///
/// Using generics here with serde derive macros causes some wierd behaviour with the compiler, so
/// as suggested by [this `serde` issue][serde-issue], just adding `#[serde(bound = "")]`
/// everywhere the generic is declared fixes the bug.
/// Hopefully this get fixed at some point so we can make this more pleasant to read...
///
/// [serde-issue]: https://github.com/serde-rs/serde/issues/1296
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowHandle<H>(#[serde(bound = "")] pub H)
where
    H: Handle;

/// Handle for testing purposes
pub type MockHandle = i32;
impl Handle for MockHandle {}

/// Store Window information.
// We allow this as we're not managing state directly. This could be refactored in the future.
// TODO: Refactor floating
#[allow(clippy::struct_excessive_bools)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Window<H: Handle> {
    #[serde(bound = "")]
    pub handle: WindowHandle<H>,
    #[serde(bound = "")]
    pub transient: Option<WindowHandle<H>>,
    visible: bool,
    pub can_resize: bool,
    is_floating: bool,
    pub(crate) must_float: bool,
    floating: Option<Xyhw>,
    pub never_focus: bool,
    pub urgent: bool,
    pub debugging: bool,
    pub name: Option<String>,
    pub legacy_name: Option<String>,
    pub pid: Option<u32>,
    pub r#type: WindowType,
    pub tag: Option<TagId>,
    pub border: i32,
    pub margin: Margins,
    pub margin_multiplier: f32,
    pub states: Vec<WindowState>,
    pub requested: Option<Xyhw>,
    pub normal: Xyhw,
    pub start_loc: Option<Xyhw>,
    pub container_size: Option<Xyhw>,
    pub strut: Option<Xyhw>,
    // Two strings that are within a XClassHint, kept separate for simpler comparing.
    pub res_name: Option<String>,
    pub res_class: Option<String>,
}

impl<H: Handle> Window<H> {
    #[must_use]
    pub fn new(h: WindowHandle<H>, name: Option<String>, pid: Option<u32>) -> Self {
        Self {
            handle: h,
            transient: None,
            visible: false,
            can_resize: true,
            is_floating: false,
            must_float: false,
            debugging: false,
            never_focus: false,
            urgent: false,
            name,
            pid,
            legacy_name: None,
            r#type: WindowType::Normal,
            tag: None,
            border: 1,
            margin: Margins::new(10),
            margin_multiplier: 1.0,
            states: vec![],
            normal: XyhwBuilder::default().into(),
            requested: None,
            floating: None,
            start_loc: None,
            container_size: None,
            strut: None,
            res_name: None,
            res_class: None,
        }
    }

    pub fn set_visible(&mut self, value: bool) {
        self.visible = value;
    }

    #[must_use]
    pub fn visible(&self) -> bool {
        self.visible
            || self.r#type == WindowType::Menu
            || self.r#type == WindowType::Splash
            || self.r#type == WindowType::Toolbar
    }

    pub fn set_floating(&mut self, value: bool) {
        if !self.is_floating && value && self.floating.is_none() {
            // NOTE: We float relative to the normal position.
            self.reset_float_offset();
        }
        self.is_floating = value;
    }

    #[must_use]
    pub fn floating(&self) -> bool {
        self.is_floating || self.must_float()
    }

    #[must_use]
    pub const fn get_floating_offsets(&self) -> Option<Xyhw> {
        self.floating
    }

    pub fn reset_float_offset(&mut self) {
        let mut new_value = Xyhw::default();
        new_value.clear_minmax();
        self.floating = Some(new_value);
    }

    pub fn set_floating_offsets(&mut self, value: Option<Xyhw>) {
        self.floating = value;
        if let Some(value) = &mut self.floating {
            value.clear_minmax();
        }
    }

    pub fn set_floating_exact(&mut self, value: Xyhw) {
        let mut new_value = value - self.normal;
        new_value.clear_minmax();
        self.floating = Some(new_value);
    }

    #[must_use]
    pub fn is_fullscreen(&self) -> bool {
        self.states.contains(&WindowState::Fullscreen)
    }

    #[must_use]
    pub fn is_maximized(&self) -> bool {
        self.states.contains(&WindowState::Maximized)
    }

    #[must_use]
    pub fn is_sticky(&self) -> bool {
        self.states.contains(&WindowState::Sticky)
    }

    #[must_use]
    pub fn must_float(&self) -> bool {
        self.must_float
            || self.transient.is_some()
            || !self.is_managed()
            || self.r#type == WindowType::Splash
    }
    #[must_use]
    pub fn can_move(&self) -> bool {
        self.is_managed()
    }
    #[must_use]
    pub fn can_resize(&self) -> bool {
        self.can_resize && self.is_managed()
    }

    #[must_use]
    pub fn can_focus(&self) -> bool {
        !self.never_focus && self.is_managed() && self.visible()
    }

    pub fn set_width(&mut self, width: i32) {
        self.normal.set_w(width);
    }

    pub fn set_height(&mut self, height: i32) {
        self.normal.set_h(height);
    }

    pub fn apply_margin_multiplier(&mut self, value: f32) {
        self.margin_multiplier = value.abs();
        if value < 0 as f32 {
            tracing::warn!(
                "Negative margin multiplier detected. Will be applied as absolute: {:?}",
                self.margin_multiplier()
            );
        };
    }

    #[must_use]
    pub const fn margin_multiplier(&self) -> f32 {
        self.margin_multiplier
    }

    #[must_use]
    pub fn width(&self) -> i32 {
        let mut value;
        if self.is_fullscreen() {
            value = self.normal.w();
        } else if self.floating() && self.floating.is_some() && !self.is_maximized() {
            let relative = self.normal + self.floating.unwrap_or_default();
            value = relative.w() - (self.border * 2);
        } else {
            value = self.normal.w()
                - (((self.margin.left + self.margin.right) as f32) * self.margin_multiplier) as i32
                - (self.border * 2);
        }
        let limit = match self.requested {
            Some(requested) if requested.minw() > 0 && self.floating() => requested.minw(),
            _ => 100,
        };
        if value < limit && self.is_managed() {
            value = limit;
        }
        value
    }

    #[must_use]
    pub fn height(&self) -> i32 {
        let mut value;
        if self.is_fullscreen() {
            value = self.normal.h();
        } else if self.floating() && self.floating.is_some() && !self.is_maximized() {
            let relative = self.normal + self.floating.unwrap_or_default();
            value = relative.h() - (self.border * 2);
        } else {
            value = self.normal.h()
                - (((self.margin.top + self.margin.bottom) as f32) * self.margin_multiplier) as i32
                - (self.border * 2);
        }
        let limit = match self.requested {
            Some(requested) if requested.minh() > 0 && self.floating() => requested.minh(),
            _ => 100,
        };
        if value < limit && self.is_managed() {
            value = limit;
        }
        value
    }

    pub fn set_x(&mut self, x: i32) {
        self.normal.set_x(x);
    }
    pub fn set_y(&mut self, y: i32) {
        self.normal.set_y(y);
    }

    #[must_use]
    pub fn border(&self) -> i32 {
        if self.is_fullscreen() {
            0
        } else {
            self.border
        }
    }

    #[must_use]
    pub fn x(&self) -> i32 {
        if self.is_fullscreen() {
            self.normal.x()
        } else if self.floating() && self.floating.is_some() && !self.is_maximized() {
            let relative = self.normal + self.floating.unwrap_or_default();
            relative.x()
        } else {
            self.normal.x() + (self.margin.left as f32 * self.margin_multiplier) as i32
        }
    }

    #[must_use]
    pub fn y(&self) -> i32 {
        if self.is_fullscreen() {
            self.normal.y()
        } else if self.floating() && self.floating.is_some() && !self.is_maximized() {
            let relative = self.normal + self.floating.unwrap_or_default();
            relative.y()
        } else {
            self.normal.y() + (self.margin.top as f32 * self.margin_multiplier) as i32
        }
    }

    #[must_use]
    pub fn calculated_xyhw(&self) -> Xyhw {
        XyhwBuilder {
            h: self.height(),
            w: self.width(),
            x: self.x(),
            y: self.y(),
            ..XyhwBuilder::default()
        }
        .into()
    }

    #[must_use]
    pub fn exact_xyhw(&self) -> Xyhw {
        if self.floating() && self.floating.is_some() {
            self.normal + self.floating.unwrap_or_default()
        } else {
            self.normal
        }
    }

    #[must_use]
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        self.calculated_xyhw().contains_point(x, y)
    }

    pub fn tag(&mut self, tag: &TagId) {
        self.tag = Some(*tag);
    }

    #[must_use]
    pub fn has_tag(&self, tag: &TagId) -> bool {
        self.tag == Some(*tag)
    }

    pub fn untag(&mut self) {
        self.tag = None;
    }

    #[must_use]
    pub fn is_managed(&self) -> bool {
        self.r#type != WindowType::Desktop && self.r#type != WindowType::Dock
    }

    #[must_use]
    pub fn is_normal(&self) -> bool {
        self.r#type == WindowType::Normal
    }

    pub fn snap_to_workspace(&mut self, workspace: &Workspace) -> bool {
        self.set_floating(false);

        // We are reparenting.
        if self.tag != workspace.tag {
            self.tag = workspace.tag;
            let mut offset = self.get_floating_offsets().unwrap_or_default();
            let mut start_loc = self.start_loc.unwrap_or_default();
            let x = offset.x() + self.normal.x();
            let y = offset.y() + self.normal.y();
            offset.set_x(x - workspace.xyhw.x());
            offset.set_y(y - workspace.xyhw.y());
            self.set_floating_offsets(Some(offset));

            let x = start_loc.x() + self.normal.x();
            let y = start_loc.y() + self.normal.y();
            start_loc.set_x(x - workspace.xyhw.x());
            start_loc.set_y(y - workspace.xyhw.y());
            self.start_loc = Some(start_loc);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_able_to_tag_a_window() {
        let mut subject = Window::new(WindowHandle::<MockHandle>(1), None, None);
        subject.tag(&1);
        assert!(subject.has_tag(&1), "was unable to tag the window");
    }

    #[test]
    fn should_be_able_to_untag_a_window() {
        let mut subject = Window::new(WindowHandle::<MockHandle>(1), None, None);
        subject.tag(&1);
        subject.untag();
        assert!(!subject.has_tag(&1), "was unable to untag the window");
    }
}
