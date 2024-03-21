use super::Handle;
use super::MaybeWindowHandle;
use super::Window;
use super::WindowHandle;
use super::WindowState;
use super::WindowType;
use super::Xyhw;
use crate::models::{Margins, XyhwChange};

type MaybeName = Option<String>;

#[derive(Debug, Clone)]
pub struct WindowChange<H: Handle> {
    pub handle: WindowHandle<H>,
    pub transient: Option<MaybeWindowHandle<H>>,
    pub never_focus: Option<bool>,
    pub urgent: Option<bool>,
    pub name: Option<MaybeName>,
    pub r#type: Option<WindowType>,
    pub floating: Option<XyhwChange>,
    pub strut: Option<XyhwChange>,
    pub requested: Option<Xyhw>,
    pub states: Option<Vec<WindowState>>,
}

impl<H: Handle> WindowChange<H> {
    #[must_use]
    pub const fn new(h: WindowHandle<H>) -> Self {
        Self {
            handle: h,
            transient: None,
            never_focus: None,
            name: None,
            r#type: None,
            urgent: None,
            floating: None,
            strut: None,
            requested: None,
            states: None,
        }
    }

    pub fn update(self, window: &mut Window<H>, container: Option<Xyhw>) -> bool {
        let mut changed = false;
        if let Some(trans) = &self.transient {
            let changed_trans = window.transient.is_none() || &window.transient != trans;
            changed = changed || changed_trans;
            window.transient = *trans;
        }
        if let Some(name) = &self.name {
            let changed_name = window.name.is_none() || &window.name != name;
            changed = changed || changed_name;
            window.name = name.clone();
        }
        if let Some(nf) = self.never_focus {
            let changed_nf = window.never_focus != nf;
            changed = changed || changed_nf;
            window.never_focus = nf;
        }
        if let Some(urgent) = self.urgent {
            let changed_urgent = window.urgent != urgent;
            changed = changed || changed_urgent;
            window.urgent = urgent;
        }
        if let Some(mut floating_change) = self.floating {
            // Reposition if dialog or modal.
            if let Some(outer) = container {
                let mut xyhw = Xyhw::default();
                floating_change.update(&mut xyhw);
                xyhw.center_relative(outer, window.border);
                floating_change.x = Some(xyhw.x());
                floating_change.y = Some(xyhw.y());
            }
            let changed_floating = floating_change.update_window_floating(window);
            changed = changed || changed_floating;
        }
        if let Some(strut) = self.strut {
            let changed_strut = strut.update_window_strut(window);
            changed = changed || changed_strut;
        }
        if let Some(requested) = self.requested {
            window.requested = Some(requested);
        }
        if let Some(r#type) = &self.r#type {
            let changed_type = &window.r#type != r#type;
            changed = changed || changed_type;
            window.r#type = r#type.clone();
            if !window.is_managed() {
                window.border = 0;
                window.margin = Margins::new(0);
            }
        }
        if let Some(states) = self.states {
            changed = true;
            window.states = states;
        }
        changed
    }
}
