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
        }
    }

    pub fn update(&self, window: &mut Window) {
        if let Some(trans) = &self.transient {
            window.transient = trans.clone();
        }
        if let Some(name) = &self.name {
            window.name = name.clone();
        }
        if let Some(nf) = self.never_focus {
            window.never_focus = nf;
        }
        if let Some(floating) = self.floating {
            window.floating = Some(floating);
        }
        if let Some(type_) = &self.type_ {
            window.type_ = type_.clone();
            if window.type_ == WindowType::Dock {
                window.border = 0;
                window.margin = 0;
            }
        }
    }
}
