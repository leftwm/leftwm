use super::config::Config;
use super::config::Keybind;
use super::utils::xkeysym_lookup;
use super::Command;
use super::ModMask;
use super::XKeysym;
use std::collections::HashMap;
use std::marker::PhantomData;
use x11_dl::xlib;

pub struct CommandBuilder<C, CMD> {
    keybinds: HashMap<(ModMask, XKeysym), Keybind<CMD>>,
    marker: PhantomData<C>,
}

impl<C: Config<CMD>, CMD> CommandBuilder<C, CMD> {
    pub fn new(config: &impl Config<CMD>) -> Self {
        let binds = config.mapped_bindings();
        let mut lookup = HashMap::new();
        for b in binds {
            if let Some(key) = xkeysym_lookup::into_keysym(&b.key) {
                let id = (xkeysym_lookup::into_modmask(&b.modifier), key);
                lookup.insert(id, b);
            }
        }
        CommandBuilder {
            keybinds: lookup,
            marker: PhantomData,
        }
    }

    pub fn find_keybind_for(&self, m: ModMask, key: XKeysym) -> Option<&Keybind<CMD>> {
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
    ) -> Option<(&Command<CMD>, Option<&str>)> {
        let keybind = self.find_keybind_for(mask, key);
        match keybind {
            Some(bind) => {
                let cmd = &bind.command;
                let val = bind.value.as_deref();
                Some((cmd, val))
            }
            None => None,
        }
    }
}
