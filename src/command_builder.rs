use super::config::Config;
use super::config::Keybind;
use super::utils::xkeysym_lookup;
use super::Command;
use super::ModMask;
use super::XKeysym;
use std::collections::HashMap;
use x11_dl::xlib;

pub struct CommandBuilder {
    keybinds: HashMap<(ModMask, XKeysym), Keybind>,
}

impl CommandBuilder {
    pub fn new(config: &impl Config) -> Self {
        let binds = config.mapped_bindings();
        let mut lookup = HashMap::new();
        for b in binds {
            if let Some(key) = xkeysym_lookup::into_keysym(&b.key) {
                let id = (xkeysym_lookup::into_modmask(&b.modifier), key);
                lookup.insert(id, b);
            }
        }
        Self { keybinds: lookup }
    }

    pub fn find_keybind_for(&self, m: ModMask, key: XKeysym) -> Option<&Keybind> {
        let mut mask = m;
        mask &= !(xlib::Mod2Mask | xlib::LockMask);
        mask &= xlib::ShiftMask
            | xlib::ControlMask
            | xlib::Mod1Mask
            | xlib::Mod3Mask
            | xlib::Mod4Mask
            | xlib::Mod5Mask;
        let id = (mask, key);
        self.keybinds.get(&id)
    }

    //Command((Command, Option<String>)),
    pub fn xkeyevent(
        &self,
        mask: ModMask,
        key: XKeysym,
        //event: XKeyEvent,
    ) -> Option<(Command, Option<String>)> {
        let keybind = self.find_keybind_for(mask, key);
        match keybind {
            Some(bind) => {
                let cmd = bind.command.clone();
                let val = bind.value.clone();
                Some((cmd, val))
            }
            None => None,
        }
    }
}
