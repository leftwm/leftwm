use std::os::raw::c_uint;
use x11_dl::xlib;

pub type ModMask = c_uint;
pub type Button = c_uint;

#[must_use]
pub fn into_modmask(keys: &[String]) -> ModMask {
    let mut mask = 0;
    for s in keys {
        mask |= into_mod(s);
    }
    // clean the mask
    mask &= !(xlib::Mod2Mask | xlib::LockMask);
    mask & (xlib::ShiftMask
        | xlib::ControlMask
        | xlib::Mod1Mask
        | xlib::Mod3Mask
        | xlib::Mod4Mask
        | xlib::Mod5Mask)
}

#[must_use]
pub fn into_mod(key: &str) -> ModMask {
    match key {
        "None" => xlib::AnyModifier,
        "Shift" => xlib::ShiftMask,
        "Control" => xlib::ControlMask,
        "Mod1" | "Alt" => xlib::Mod1Mask,
        // "Mod2" => xlib::Mod2Mask,     // NOTE: we are ignoring the state of Numlock
        // "NumLock" => xlib::Mod2Mask,  // this is left here as a reminder
        "Mod3" => xlib::Mod3Mask,
        "Mod4" | "Super" => xlib::Mod4Mask,
        "Mod5" => xlib::Mod5Mask,
        _ => 0,
    }
}
