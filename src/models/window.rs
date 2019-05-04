use super::WindowType;
use crate::config::ThemeSetting;
use crate::models::XYHWBuilder;
use crate::models::XYHW;
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
    visable: bool,
    is_floating: bool,
    pub never_focus: bool,
    pub name: Option<String>,
    pub type_: WindowType,
    pub tags: Vec<String>,
    pub border: i32,
    pub margin: i32,
    pub fullscreen: bool,
    pub normal: XYHW,
    pub floating: Option<XYHW>,
    pub start_loc: Option<(i32, i32)>,
}

impl Window {
    pub fn new(h: WindowHandle, name: Option<String>) -> Window {
        Window {
            handle: h,
            transient: None,
            visable: false,
            is_floating: false,
            never_focus: false,
            name,
            type_: WindowType::Normal,
            tags: Vec::new(),
            border: 1,
            margin: 10,
            fullscreen: false,
            normal: XYHWBuilder::default().into(),
            floating: None,
            start_loc: None,
        }
    }

    pub fn update_for_theme(&mut self, theme: &ThemeSetting) {
        if self.type_ == WindowType::Normal {
            self.margin = theme.margin as i32;
            self.border = theme.border_width as i32;
        } else {
            self.margin = 0;
            self.border = 0;
        }
    }

    pub fn set_visable(&mut self, value: bool) {
        self.visable = value;
    }
    pub fn visable(&self) -> bool {
        self.visable || self.floating()
    }

    pub fn set_floating(&mut self, value: bool) {
        self.is_floating = value;
    }
    pub fn floating(&self) -> bool {
        self.is_floating || self.must_float()
    }
    pub fn must_float(&self) -> bool {
        self.transient.is_some()
            || self.type_ == WindowType::Dock
            || self.type_ == WindowType::Splash
            || self.fullscreen
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

    pub fn width(&self) -> i32 {
        if self.floating() && self.floating.is_some() {
            self.floating.unwrap().w()
        } else {
            self.normal.w() - (self.margin * 2) - (self.border * 2)
        }
    }
    pub fn height(&self) -> i32 {
        if self.floating() && self.floating.is_some() {
            self.floating.unwrap().h()
        } else {
            self.normal.h() - (self.margin * 2) - (self.border * 2)
        }
    }

    pub fn set_x(&mut self, x: i32) {
        self.normal.set_x(x)
    }
    pub fn set_y(&mut self, y: i32) {
        self.normal.set_y(y)
    }

    pub fn x(&self) -> i32 {
        if self.floating() && self.floating.is_some() {
            self.floating.unwrap().x()
        } else {
            self.normal.x() + self.margin
        }
    }

    pub fn y(&self) -> i32 {
        if self.floating() && self.floating.is_some() {
            self.floating.unwrap().y()
        } else {
            self.normal.y() + self.margin
        }
    }

    pub fn tag(&mut self, tag: String) {
        if tag == "" {
            return;
        }
        for t in &self.tags {
            if t == &tag {
                return;
            }
        }
        self.tags.push(tag);
    }

    pub fn clear_tags(&mut self) {
        self.tags = vec![];
    }

    pub fn has_tag(&self, tag: String) -> bool {
        self.tags.contains(&tag)
    }

    pub fn untag(&mut self, tag: String) {
        let mut new_tags: Vec<String> = Vec::new();
        for t in &self.tags {
            if t != &tag {
                new_tags.push(t.clone())
            }
        }
        self.tags = new_tags;
    }
}

#[test]
fn should_be_able_to_tag_a_window() {
    let mut subject = Window::new(WindowHandle::MockHandle(1), None);
    subject.tag("test".to_string());
    assert!(
        subject.has_tag("test".to_string()),
        "was unable to tag the window"
    );
}

#[test]
fn should_be_able_to_untag_a_window() {
    let mut subject = Window::new(WindowHandle::MockHandle(1), None);
    subject.tag("test".to_string());
    subject.untag("test".to_string());
    assert!(
        subject.has_tag("test".to_string()) == false,
        "was unable to untag the window"
    );
}
