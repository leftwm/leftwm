use super::MaybeWindowHandle;
use super::Window;
use super::WindowHandle;
use super::WindowState;
use super::WindowType;
use crate::models::{Margins, XyhwChange};

type MaybeName = Option<String>;

#[derive(Debug, Clone)]
pub struct WindowChange {
    pub handle: WindowHandle,
    pub transient: Option<MaybeWindowHandle>,
    pub never_focus: Option<bool>,
    pub name: Option<MaybeName>,
    pub type_: Option<WindowType>,
    pub floating: Option<XyhwChange>,
    pub strut: Option<XyhwChange>,
    pub states: Option<Vec<WindowState>>,
}

impl WindowChange {
    #[must_use]
    pub fn new(h: WindowHandle) -> WindowChange {
        WindowChange {
            handle: h,
            transient: None,
            never_focus: None,
            name: None,
            type_: None,
            floating: None,
            strut: None,
            states: None,
        }
    }

    pub fn update(self, window: &mut Window) -> bool {
        let mut changed = false;
        if let Some(trans) = &self.transient {
            let changed_trans = window.transient.is_none() || &window.transient != trans;
            //if changed_trans {
            //    warn!("CHANGED: trans");
            //}
            changed = changed || changed_trans;
            window.transient = *trans;
        }
        if let Some(name) = &self.name {
            let changed_name = window.name.is_none() || &window.name != name;
            //if changed_name {
            //    warn!("CHANGED: name");
            //}
            changed = changed || changed_name;
            window.name = name.clone();
        }
        if let Some(nf) = self.never_focus {
            let changed_nf = window.never_focus != nf;
            //if changed_nf {
            //    warn!("CHANGED: nf");
            //}
            changed = changed || changed_nf;
            window.never_focus = nf;
        }
        if let Some(floating_change) = self.floating {
            let changed_floating = floating_change.update_window_floating(window);
            //if changed_floating {
            //    warn!("CHANGED: floating");
            //}
            changed = changed || changed_floating;
        }
        if let Some(strut) = self.strut {
            let changed_strut = strut.update_window_strut(window);
            //////if changed_strut {
            //////    warn!("CHANGED: strut");
            //////}
            changed = changed || changed_strut;
        }
        if let Some(type_) = &self.type_ {
            let changed_type = &window.type_ != type_;
            //if changed_type {
            //    warn!("CHANGED: type");
            //}
            changed = changed || changed_type;
            window.type_ = type_.clone();
            if window.is_unmanaged() {
                window.border = 0;
                window.margin = Margins::new(0);
            }
        }
        if let Some(states) = self.states {
            //warn!("CHANGED: state");
            changed = true;
            window.set_states(states);
        }
        changed
    }
}
