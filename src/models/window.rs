use super::WindowState;
use super::WindowType;
use crate::config::ThemeSetting;
use crate::models::Margins;
use crate::models::TagId;
use crate::models::Xyhw;
use crate::models::XyhwBuilder;
use serde::{Deserialize, Serialize};
use x11_dl::xlib;

type MockHandle = i32;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum WindowHandle {
    MockHandle(MockHandle),
    XlibHandle(xlib::Window),
}

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
    pub type_: WindowType,
    pub tags: Vec<TagId>,
    pub border: i32,
    pub margin: Margins,
    states: Vec<WindowState>,
    pub normal: Xyhw,
    pub start_loc: Option<Xyhw>,
    pub container_size: Option<Xyhw>,
    pub strut: Option<Xyhw>,
}

impl Window {
    pub fn new(h: WindowHandle, name: Option<String>) -> Window {
        Window {
            handle: h,
            transient: None,
            visible: false,
            is_floating: false,
            debugging: false,
            never_focus: false,
            name,
            type_: WindowType::Normal,
            tags: Vec::new(),
            border: 1,
            margin: Margins::Int(10),
            states: vec![],
            normal: XyhwBuilder::default().into(),
            floating: None,
            start_loc: None,
            container_size: None,
            strut: None,
        }
    }

    pub fn update_for_theme(&mut self, theme: &ThemeSetting) {
        if self.type_ == WindowType::Normal {
            self.margin = theme.margin.clone();
            self.border = theme.border_width;
        } else {
            self.margin = Margins::Int(0);
            self.border = 0;
        }
    }

    pub fn set_visible(&mut self, value: bool) {
        self.visible = value;
    }

    pub fn visible(&self) -> bool {
        self.visible
            || self.type_ == WindowType::Dock
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

    pub fn floating(&self) -> bool {
        self.is_floating || self.must_float()
    }

    pub fn get_floating_offsets(&self) -> Option<Xyhw> {
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

    pub fn is_fullscreen(&self) -> bool {
        self.states.contains(&WindowState::Fullscreen)
    }
    pub fn must_float(&self) -> bool {
        self.transient.is_some()
            || self.type_ == WindowType::Dock
            || self.type_ == WindowType::Splash
            || self.is_fullscreen()
    }
    pub fn can_move(&self) -> bool {
        self.type_ != WindowType::Dock
    }
    pub fn can_resize(&self) -> bool {
        self.type_ != WindowType::Dock
    }

    pub fn set_width(&mut self, width: i32) {
        self.normal.set_w(width)
    }
    pub fn set_height(&mut self, height: i32) {
        self.normal.set_h(height)
    }
    pub fn set_states(&mut self, states: Vec<WindowState>) {
        self.states = states;
    }

    pub fn width(&self) -> i32 {
        let mut value;
        if self.is_fullscreen() {
            value = self.normal.w();
        } else if self.floating() && self.floating.is_some() {
            let relative = self.normal + self.floating.unwrap();
            value = relative.w()
                - (self.margin.clone().left() + self.margin.clone().right())
                - (self.border * 2);
        } else {
            value = self.normal.w()
                - (self.margin.clone().left() + self.margin.clone().right())
                - (self.border * 2);
        }
        if value < 100 && self.type_ != WindowType::Dock {
            value = 100
        }
        value
    }
    pub fn height(&self) -> i32 {
        let mut value;
        if self.is_fullscreen() {
            value = self.normal.h();
        } else if self.floating() && self.floating.is_some() {
            let relative = self.normal + self.floating.unwrap();
            value = relative.h()
                - (self.margin.clone().top() + self.margin.clone().bottom())
                - (self.border * 2);
        } else {
            value = self.normal.h()
                - (self.margin.clone().top() + self.margin.clone().bottom())
                - (self.border * 2);
        }
        if value < 100 && self.type_ != WindowType::Dock {
            value = 100
        }
        value
    }

    pub fn set_x(&mut self, x: i32) {
        self.normal.set_x(x)
    }
    pub fn set_y(&mut self, y: i32) {
        self.normal.set_y(y)
    }

    pub fn border(&self) -> i32 {
        if self.is_fullscreen() {
            0
        } else {
            self.border
        }
    }

    pub fn x(&self) -> i32 {
        if self.is_fullscreen() {
            return self.normal.x();
        }
        if self.floating() && self.floating.is_some() {
            let relative = self.normal + self.floating.unwrap();
            relative.x() + self.margin.clone().left()
        } else {
            self.normal.x() + self.margin.clone().left()
        }
    }

    pub fn y(&self) -> i32 {
        if self.is_fullscreen() {
            return self.normal.y();
        }
        if self.floating() && self.floating.is_some() {
            let relative = self.normal + self.floating.unwrap();
            relative.y() + self.margin.clone().bottom()
        } else {
            self.normal.y() + self.margin.clone().bottom()
        }
    }

    pub fn calculated_xyhw(&self) -> Xyhw {
        XyhwBuilder {
            h: self.height(),
            w: self.width(),
            x: self.x(),
            y: self.y(),
            ..Default::default()
        }
        .into()
    }

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

    pub fn has_tag(&self, tag: &str) -> bool {
        let t = tag.to_owned();
        self.tags.contains(&t)
    }

    pub fn untag(&mut self, tag: &str) {
        let mut new_tags: Vec<TagId> = Vec::new();
        for t in &self.tags {
            if t != tag {
                new_tags.push(t.clone())
            }
        }
        self.tags = new_tags;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_able_to_tag_a_window() {
        let mut subject = Window::new(WindowHandle::MockHandle(1), None);
        subject.tag("test");
        assert!(subject.has_tag("test"), "was unable to tag the window");
    }

    #[test]
    fn should_be_able_to_untag_a_window() {
        let mut subject = Window::new(WindowHandle::MockHandle(1), None);
        subject.tag("test");
        subject.untag("test");
        assert!(!subject.has_tag("test"), "was unable to untag the window");
    }
}
