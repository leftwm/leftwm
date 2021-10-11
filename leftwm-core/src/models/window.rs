//! Window Information
#![allow(clippy::module_name_repetitions)]
use super::WindowState;
use super::WindowType;
use crate::config::Config;
use crate::models::xyhw_change::XyhwChange;
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

/// Store Window information.
// We allow this as we're not managing state directly. This could be refactored in the future.
// TODO: Refactor floating
#[allow(clippy::struct_excessive_bools)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Window {
    pub handle: WindowHandle,
    pub transient: Option<WindowHandle>,
    visible: bool,
    is_floating: bool,
    floating: Option<Xyhw>,
    pub never_focus: bool,
    pub debugging: bool,
    pub name: Option<String>,
    pub pid: Option<u32>,
    pub type_: WindowType,
    pub tags: Vec<TagId>,
    pub border: i32,
    pub margin: Margins,
    pub margin_multiplier: f32,
    states: Vec<WindowState>,
    pub requested: Option<XyhwChange>,
    pub normal: Xyhw,
    pub start_loc: Option<Xyhw>,
    pub container_size: Option<Xyhw>,
    pub strut: Option<Xyhw>,
}

impl Window {
    #[must_use]
    pub fn new(h: WindowHandle, name: Option<String>, pid: Option<u32>) -> Self {
        Self {
            handle: h,
            transient: None,
            visible: false,
            is_floating: false,
            debugging: false,
            never_focus: false,
            name,
            pid,
            type_: WindowType::Normal,
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
        }
    }

    pub fn update_for_theme(&mut self, config: &impl Config) {
        if self.type_ == WindowType::Normal {
            self.margin = config.margin();
            self.border = config.border_width();
        } else {
            self.margin = Margins::new(0);
            self.border = 0;
        }
    }

    pub fn set_visible(&mut self, value: bool) {
        self.visible = value;
    }

    #[must_use]
    pub fn visible(&self) -> bool {
        self.visible
            || self.type_ == WindowType::Menu
            || self.type_ == WindowType::Splash
            || self.type_ == WindowType::Dialog
            || self.type_ == WindowType::Toolbar
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
        self.transient.is_some() || self.is_unmanaged() || self.type_ == WindowType::Splash
    }
    #[must_use]
    pub fn can_move(&self) -> bool {
        !self.is_unmanaged()
    }
    #[must_use]
    pub fn can_resize(&self) -> bool {
        !self.is_unmanaged()
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

    pub fn set_requested(&mut self, change: XyhwChange) {
        self.requested = Some(change);
    }

    pub fn get_requested(&mut self) -> Option<XyhwChange> {
        self.requested
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
        if value < 100 && !self.is_unmanaged() {
            value = 100;
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
        if value < 100 && !self.is_unmanaged() {
            value = 100;
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
            return self.normal.x();
        }
        if self.floating() && self.floating.is_some() {
            let relative = self.normal + self.floating.unwrap_or_default();
            relative.x()
        } else {
            self.normal.x() + (self.margin.left as f32 * self.margin_multiplier) as i32
        }
    }

    #[must_use]
    pub fn y(&self) -> i32 {
        if self.is_fullscreen() {
            return self.normal.y();
        }
        if self.floating() && self.floating.is_some() {
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
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        self.calculated_xyhw().contains_point(x, y)
    }

    pub fn tag(&mut self, tag: &str) {
        if tag.is_empty() {
            return;
        }
        for t in &self.tags {
            if t == tag {
                return;
            }
        }
        self.tags.push(tag.to_string());
    }

    pub fn clear_tags(&mut self) {
        self.tags = vec![];
    }

    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        let t = tag.to_owned();
        self.tags.contains(&t)
    }

    pub fn untag(&mut self, tag: &str) {
        let mut new_tags: Vec<TagId> = Vec::new();
        for t in &self.tags {
            if t != tag {
                new_tags.push(t.clone());
            }
        }
        self.tags = new_tags;
    }

    #[must_use]
    pub fn is_unmanaged(&self) -> bool {
        self.type_ == WindowType::Desktop || self.type_ == WindowType::Dock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_able_to_tag_a_window() {
        let mut subject = Window::new(WindowHandle::MockHandle(1), None, None);
        subject.tag("test");
        assert!(subject.has_tag("test"), "was unable to tag the window");
    }

    #[test]
    fn should_be_able_to_untag_a_window() {
        let mut subject = Window::new(WindowHandle::MockHandle(1), None, None);
        subject.tag("test");
        subject.untag("test");
        assert!(!subject.has_tag("test"), "was unable to untag the window");
    }
}
