use super::Window;
use super::WindowHandle;
use super::WindowType;
use crate::models::XYHW;

type MaybeWindowHandle = Option<WindowHandle>;
type MaybeName = Option<String>;

#[derive(Debug, Clone)]
pub struct WindowChange {
    pub handle: WindowHandle,
    pub transient: Option<MaybeWindowHandle>,
    pub never_focus: Option<bool>,
    pub name: Option<MaybeName>,
    pub type_: Option<WindowType>,
    pub floating: Option<XYHW>,
    pub toggle_fullscreen: Option<bool>,
    pub set_fullscreen: Option<bool>,
}

impl WindowChange {
    pub fn new(h: WindowHandle) -> WindowChange {
        WindowChange {
            handle: h,
            transient: None,
            never_focus: None,
            name: None,
            type_: None,
            floating: None,
            toggle_fullscreen: None,
            set_fullscreen: None,
        }
    }

    pub fn update(&self, window: &mut Window) -> bool {
        let mut changed = false;
        if let Some(trans) = &self.transient {
            changed = window.transient.is_none() || &window.transient != trans;
            window.transient = trans.clone();
        }
        if let Some(name) = &self.name {
            changed = changed || window.name.is_none() || &window.name != name;
            window.name = name.clone();
        }
        if let Some(nf) = self.never_focus {
            changed = changed || window.never_focus != nf;
            window.never_focus = nf;
        }
        if let Some(floating) = self.floating {
            changed = changed || window.floating.is_none() || window.floating.unwrap() != floating;
            window.floating = Some(floating);
        }
        if let Some(type_) = &self.type_ {
            changed = changed || &window.type_ != type_;
            window.type_ = type_.clone();
            if window.type_ == WindowType::Dock {
                window.border = 0;
                window.margin = 0;
            }
        }
        if self.toggle_fullscreen.is_some() {
            window.fullscreen = !window.fullscreen;
            return true;
        }
        if let Some(fs) = self.set_fullscreen {
            changed = changed || window.fullscreen != fs;
            window.fullscreen = fs;
        }
        changed
    }
}
