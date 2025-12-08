use crate::drawing::border::NormalisedColor;
use leftwm_core::models::FocusBehaviour;

#[derive(Debug)]
pub struct LeftwmConfig {
    pub focus_behavior: FocusBehaviour,
    pub sloppy_mouse_follows_focus: bool,
    pub borders: BorderConfig,
}

#[derive(Clone, Copy, Debug)]
pub struct BorderConfig {
    pub border_width: i32,
    pub default_border_color: NormalisedColor,
    pub floating_border_color: NormalisedColor,
    pub focused_border_color: NormalisedColor,
}
