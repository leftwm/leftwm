//! Window Information
#![allow(clippy::module_name_repetitions)]
use super::WindowState;
use super::WindowType;
use crate::models::Margins;
use crate::models::TagId;
use crate::models::Xyhw;
use crate::models::XyhwBuilder;
use serde::{Deserialize, Serialize};
use x11_dl::xlib;

type MockHandle = i32;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum WindowHandle {
    MockHandle(MockHandle),
    XlibHandle(xlib::Window),
}

impl std::convert::From<xlib::Window> for WindowHandle {
    fn from(window: xlib::Window) -> Self {
        WindowHandle::XlibHandle(window)
    }
}

impl WindowHandle {
    pub fn xlib_handle(self) -> Option<xlib::Window> {
        match self {
            WindowHandle::MockHandle(_) => None,
            WindowHandle::XlibHandle(h) => Some(h),
        }
    }
}

/// Store Window information.
// We allow this as we're not managing state directly. This could be refactored in the future.
// TODO: Refactor floating
#[allow(clippy::struct_excessive_bools)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Window {
    pub handle: WindowHandle,
    pub transient: Option<WindowHandle>,
    visible: bool,
    pub can_resize: bool,
    is_floating: bool,
    pub(crate) must_float: bool,
    floating: Option<Xyhw>,
    pub never_focus: bool,
    pub debugging: bool,
    pub name: Option<String>,
    pub legacy_name: Option<String>,
    pub pid: Option<u32>,
    pub r#type: WindowType,
    pub tags: Vec<TagId>,
    pub border: i32,
    pub margin: Margins,
    pub margin_multiplier: f32,
    states: Vec<WindowState>,
    pub requested: Option<Xyhw>,
    pub normal: Xyhw,
    pub start_loc: Option<Xyhw>,
    pub container_size: Option<Xyhw>,
    pub strut: Option<Xyhw>,
    // Two strings that are within a XClassHint, kept separate for simpler comparing.
    pub res_name: Option<String>,
    pub res_class: Option<String>,
}

impl Window {
    #[must_use]
    pub fn new(h: WindowHandle, name: Option<String>, pid: Option<u32>) -> Self {
        Self {
            handle: h,
            transient: None,
            visible: false,
            can_resize: true,
            is_floating: false,
            must_float: false,
            debugging: false,
            never_focus: false,
            name,
            pid,
            legacy_name: None,
            r#type: WindowType::Normal,
            tags: Vec::new(),
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
            //NOTE: We float relative to the normal position.
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
    pub fn is_sticky(&self) -> bool {
        self.states.contains(&WindowState::Sticky)
    }
    #[must_use]
    pub fn must_float(&self) -> bool {
        self.must_float
            || self.transient.is_some()
            || self.is_unmanaged()
            || self.r#type == WindowType::Splash
    }
    #[must_use]
    pub fn can_move(&self) -> bool {
        !self.is_unmanaged()
    }
    #[must_use]
    pub fn can_resize(&self) -> bool {
        self.can_resize && !self.is_unmanaged()
    }

    #[must_use]
    pub fn can_focus(&self) -> bool {
        !self.never_focus && !self.is_unmanaged() && self.visible()
    }

    pub fn set_width(&mut self, width: i32) {
        self.normal.set_w(width);
    }

    pub fn set_height(&mut self, height: i32) {
        self.normal.set_h(height);
    }

    pub fn set_states(&mut self, states: Vec<WindowState>) {
        self.states = states;
    }

    #[must_use]
    pub fn has_state(&self, state: &WindowState) -> bool {
        self.states.contains(state)
    }

    #[must_use]
    pub fn states(&self) -> Vec<WindowState> {
        self.states.clone()
    }

    pub fn apply_margin_multiplier(&mut self, value: f32) {
        self.margin_multiplier = value.abs();
        if value < 0 as f32 {
            log::warn!(
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
        } else if self.floating() && self.floating.is_some() {
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
        if value < limit && !self.is_unmanaged() {
            value = limit;
        }
        value
    }

    #[must_use]
    pub fn height(&self) -> i32 {
        let mut value;
        if self.is_fullscreen() {
            value = self.normal.h();
        } else if self.floating() && self.floating.is_some() {
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
        if value < limit && !self.is_unmanaged() {
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
        } else if self.floating() && self.floating.is_some() {
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
        } else if self.floating() && self.floating.is_some() {
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
        if !self.tags.contains(tag) {
            self.tags.push(*tag);
        }
    }

    pub fn clear_tags(&mut self) {
        self.tags = vec![];
    }

    #[must_use]
    pub fn has_tag(&self, tag: &TagId) -> bool {
        self.tags.contains(tag)
    }

    pub fn untag(&mut self, tag: &TagId) {
        self.tags.retain(|t| t != tag);
    }

    #[must_use]
    pub fn is_unmanaged(&self) -> bool {
        self.r#type == WindowType::Desktop || self.r#type == WindowType::Dock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_able_to_tag_a_window() {
        let mut subject = Window::new(WindowHandle::MockHandle(1), None, None);
        subject.tag(&1);
        assert!(subject.has_tag(&1), "was unable to tag the window");
    }

    #[test]
    fn should_be_able_to_untag_a_window() {
        let mut subject = Window::new(WindowHandle::MockHandle(1), None, None);
        subject.tag(&1);
        subject.untag(&1);
        assert!(!subject.has_tag(&1), "was unable to untag the window");
    }
}
