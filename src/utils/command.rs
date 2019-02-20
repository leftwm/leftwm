use super::config::Config;
use super::config::Keybind;
use super::event_queue::EventQueueItem;
use super::xkeysym_lookup;
use super::xkeysym_lookup::ModMask;
use super::xkeysym_lookup::XKeysym;
use std::collections::HashMap;
use x11_dl::xlib;
use x11_dl::xlib::XKeyEvent;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Command {
    Execute,
    CloseWindow,
    SwapTags,
    GotoTag,
    MoveToTag,
}

pub struct CommandBuilder {
    keybinds: HashMap<(ModMask, XKeysym), Keybind>,
}

impl CommandBuilder {
    pub fn new(config: &Config) -> CommandBuilder {
        let binds = config.mapped_bindings();
        let mut lookup = HashMap::new();
        for b in binds {
            if let Some(key) = xkeysym_lookup::into_keysym(&b.key) {
                let id = (xkeysym_lookup::into_modmask(&b.modifier), key);
                lookup.insert(id, b);
            }
        }
        CommandBuilder { keybinds: lookup }
    }

    pub fn find_keybind_for(&self, key: XKeysym, event: XKeyEvent) -> Option<&Keybind> {
        let mut mask = event.state;
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
    pub fn from_xkeyevent(&self, key: XKeysym, event: XKeyEvent) -> Option<EventQueueItem> {
        let keybind = self.find_keybind_for(key, event);
        match keybind {
            Some(bind) => {
                let cmd = bind.command.clone();
                let val = Some(bind.key.clone());
                Some(EventQueueItem::Command(cmd, val))
            }
            None => None,
        }
    }
}

////**
/// * returns a collection of keybinds that the magic string "modkey" as been mapped into
/// * the mod key from the config
/// */
///fn keybinds_with_mapped_modkey(config: &Config) -> Vec<Keybind>{
///    let mod_key: &String = &config.modkey.clone();
///    let old_binds :&Vec<Keybind> = &config.keybind;
///    old_binds.iter().map(|k| {
///        let mut keymap = k.clone();
///        let old_mods :&Vec<String> = &k.modifier;
///        let mods = old_mods.iter().map(|m| { if m == "modkey" { mod_key.clone() } else { m.clone() } }).collect();
///        keymap.modifier = mods;
///        keymap
///    }).collect()
///}

#[test]
fn should_be_able_to_build_a_goto_workspace_command() {
    let builder = CommandBuilder::new(&Config::default());
    let mut ev: XKeyEvent = unsafe { std::mem::zeroed() };
    let keysym = x11_dl::keysym::XK_1;
    ev.state = 8; //alt + 1
    let bind = builder.find_keybind_for(keysym, ev);
    match bind {
        Some(b) => {
            assert!(b.command == Command::GotoTag, "wrong command found");
            assert!(b.value == Some("1".to_owned()), "wrong value found");
        }
        None => {
            assert!(false, "Expected to be able to find command");
        }
    }
}

#[test]
fn should_be_able_to_build_a_goto_workspace_command_with_numlock() {
    let builder = CommandBuilder::new(&Config::default());
    let mut ev: XKeyEvent = unsafe { std::mem::zeroed() };
    let keysym = x11_dl::keysym::XK_1;
    ev.state = 24; //alt + Numlock + 1
    let bind = builder.find_keybind_for(keysym, ev);
    match bind {
        Some(b) => {
            assert!(b.command == Command::GotoTag, "wrong command found");
            assert!(b.value == Some("1".to_owned()), "wrong value found");
        }
        None => {
            assert!(false, "Expected to be able to find command");
        }
    }
}
