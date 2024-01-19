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
bitflags! {
    /// Represents the state of the mouse buttons
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Button: u8 {
        /// Used as the zero value
        const Zero = 0;
        /// Main button (left click for right-handed)
        const Button1 = 1;
        /// Middle button (pressing the scroll wheel)
        const Button2 = 1 << 1;
        /// Secondary button (right click for right-handed)
        const Button3 = 1 << 2;
        /// Scroll wheel up
        const Button4 = 1 << 3;
        /// Scroll wheel down
        const Button5 = 1 << 4;
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

        impl<'de> Visitor<'de> for ModmaskVisitor {
            type Value = ModMask;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a bitfield on 8 bits")
            }

            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ModMask::from_bits_retain(v as u16))
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

impl Serialize for Button {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.bits())
    }
}

impl<'de> Deserialize<'de> for Button {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ButtonVisitor;

        impl<'de> Visitor<'de> for ButtonVisitor {
            type Value = Button;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a bitfield on 8 bits")
            }

            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Button::from_bits_retain(v))
            }
        }

        deserializer.deserialize_u8(ButtonVisitor)
    }
}
