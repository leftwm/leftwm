use super::Window;
use super::WindowHandle;
use super::WindowType;
use crate::models::XYHW;

#[derive(Debug, Clone)]
pub struct WindowChange {
    pub handle: WindowHandle,
    pub transient: Option<Option<WindowHandle>>,
    pub name: Option<Option<String>>,
    pub type_: Option<WindowType>,
    pub floating: Option<XYHW>,
}

impl WindowChange {
    pub fn new(h: WindowHandle) -> WindowChange {
        WindowChange {
            handle: h,
            transient: None,
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
        if let Some(floating) = self.floating {
            window.floating = Some(floating);
        }
        if let Some(type_) = &self.type_ {
            window.type_ = type_.clone();
            window.margin = 0;
            window.border = 0;
        }
    }
}
