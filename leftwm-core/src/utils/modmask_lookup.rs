use bitflags::bitflags;
use serde::{de::Visitor, Deserialize, Serialize};

bitflags! {
    /// Represents the state of modifier keys
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ModMask: u16 {
        /// Used as the zero value
        const Zero = 0;
        const Any = 1;
        const Shift = 1 << 1;
        const Control = 1 << 2;
        /// Mod1
        const Alt = 1 << 3;
        /// Mod2
        const NumLock = 1 << 4;
        const Mod3 = 1 << 5;
        /// Mod4
        const Super = 1 << 6;
        const Mod5 = 1 << 7;
    }
}

/// Representation of mouse buttons
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Button {
    /// no buttons pressed
    None,
    /// Main button (left click for right-handed)
    /// Button1
    Main,
    /// Middle button (pressing the scroll wheel)
    /// Button2
    Middle,
    /// Secondary button (right click for right-handed)
    /// Button3
    Secondary,
    /// Scroll wheel up
    /// Button4
    ScrollUp,
    /// Scroll wheel down
    /// Button5
    ScrollDown,
}

impl From<u8> for Button {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Main,
            2 => Self::Middle,
            3 => Self::Secondary,
            4 => Self::ScrollUp,
            5 => Self::ScrollDown,
            _ => Self::None,
        }
    }
}

impl From<Button> for u8 {
    fn from(value: Button) -> Self {
        match value {
            Button::None => 0,
            Button::Main => 1,
            Button::Middle => 2,
            Button::Secondary => 3,
            Button::ScrollUp => 4,
            Button::ScrollDown => 5,
        }
    }
}

#[must_use]
pub fn into_modmask(keys: &[String]) -> ModMask {
    let mut mask = ModMask::Zero;
    for s in keys {
        mask |= into_mod(s);
    }
    // clean the mask
    mask.remove(ModMask::NumLock);
    mask.intersection(
        ModMask::Shift
            | ModMask::Control
            | ModMask::Alt
            | ModMask::Mod3
            | ModMask::Super
            | ModMask::Mod5,
    )
}

#[must_use]
pub fn into_mod(key: &str) -> ModMask {
    match key {
        "None" => ModMask::Any,
        "Shift" => ModMask::Shift,
        "Control" => ModMask::Control,
        "Mod1" | "Alt" => ModMask::Alt,
        // NOTE: we are ignoring the state of Numlock
        // this is left here as a reminder
        // "Mod2" | "NumLock" => ModMask::NumLock,
        "Mod3" => ModMask::Mod3,
        "Mod4" | "Super" => ModMask::Super,
        "Mod5" => ModMask::Mod5,
        _ => ModMask::Zero,
    }
}

// serde impls (derive is not working with the bitflags macro)

impl Serialize for ModMask {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u16(self.bits())
    }
}

impl<'de> Deserialize<'de> for ModMask {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ModmaskVisitor;

        impl Visitor<'_> for ModmaskVisitor {
            type Value = ModMask;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a bitfield on 8 bits")
            }

            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ModMask::from_bits_retain(u16::from(v)))
            }

            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ModMask::from_bits_retain(v))
            }
        }

        deserializer.deserialize_u16(ModmaskVisitor)
    }
}
