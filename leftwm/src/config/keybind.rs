use super::BaseCommand;
use crate::Config;
use anyhow::{ensure, Context, Result};
use leftwm_core::layouts::Layout;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Keybind {
    pub command: BaseCommand,
    #[serde(default)]
    pub value: String,
    pub modifier: Option<Modifier>,
    pub key: String,
}

macro_rules! ensure_non_empty {
    ($value:expr) => {{
        ensure!(!$value.is_empty(), "value must not be empty");
        $value
    }};
}

impl Keybind {
    pub fn try_convert_to_core_keybind(&self, config: &Config) -> Result<leftwm_core::Keybind> {
        let command = match &self.command {
            BaseCommand::Execute => {
                leftwm_core::Command::Execute(ensure_non_empty!(self.value.clone()))
            }
            BaseCommand::CloseWindow => leftwm_core::Command::CloseWindow,
            BaseCommand::SwapTags => leftwm_core::Command::SwapScreens,
            BaseCommand::SoftReload => leftwm_core::Command::SoftReload,
            BaseCommand::HardReload => leftwm_core::Command::HardReload,
            BaseCommand::ToggleScratchPad => {
                leftwm_core::Command::ToggleScratchPad(ensure_non_empty!(self.value.clone()))
            }
            BaseCommand::ToggleFullScreen => leftwm_core::Command::ToggleFullScreen,
            BaseCommand::ToggleSticky => leftwm_core::Command::ToggleSticky,
            BaseCommand::GotoTag => leftwm_core::Command::GoToTag {
                tag: usize::from_str(&self.value).context("invalid index value for GotoTag")?,
                swap: !config.disable_current_tag_swap,
            },
            BaseCommand::FloatingToTile => leftwm_core::Command::FloatingToTile,
            BaseCommand::TileToFloating => leftwm_core::Command::TileToFloating,
            BaseCommand::ToggleFloating => leftwm_core::Command::ToggleFloating,
            BaseCommand::MoveWindowUp => leftwm_core::Command::MoveWindowUp,
            BaseCommand::MoveWindowDown => leftwm_core::Command::MoveWindowDown,
            BaseCommand::MoveWindowTop => leftwm_core::Command::MoveWindowTop,
            BaseCommand::FocusNextTag => leftwm_core::Command::FocusNextTag,
            BaseCommand::FocusPreviousTag => leftwm_core::Command::FocusPreviousTag,
            BaseCommand::FocusWindow => leftwm_core::Command::FocusWindow(self.value.clone()),
            BaseCommand::FocusWindowUp => leftwm_core::Command::FocusWindowUp,
            BaseCommand::FocusWindowDown => leftwm_core::Command::FocusWindowDown,
            BaseCommand::FocusWindowTop => {
                leftwm_core::Command::FocusWindowTop(if self.value.is_empty() {
                    false
                } else {
                    bool::from_str(&self.value)
                        .context("invalid boolean value for FocusWindowTop")?
                })
            }
            BaseCommand::FocusWorkspaceNext => leftwm_core::Command::FocusWorkspaceNext,
            BaseCommand::FocusWorkspacePrevious => leftwm_core::Command::FocusWorkspacePrevious,
            BaseCommand::MoveToTag => leftwm_core::Command::SendWindowToTag {
                window: None,
                tag: usize::from_str(&self.value)
                    .context("invalid index value for SendWindowToTag")?,
            },
            BaseCommand::MoveToLastWorkspace => leftwm_core::Command::MoveWindowToLastWorkspace,
            BaseCommand::MoveWindowToNextWorkspace => {
                leftwm_core::Command::MoveWindowToNextWorkspace
            }
            BaseCommand::MoveWindowToPreviousWorkspace => {
                leftwm_core::Command::MoveWindowToPreviousWorkspace
            }
            BaseCommand::MouseMoveWindow => leftwm_core::Command::MouseMoveWindow,
            BaseCommand::NextLayout => leftwm_core::Command::NextLayout,
            BaseCommand::PreviousLayout => leftwm_core::Command::PreviousLayout,
            BaseCommand::SetLayout => leftwm_core::Command::SetLayout(
                Layout::from_str(&self.value)
                    .context("could not parse layout for command SetLayout")?,
            ),
            BaseCommand::RotateTag => leftwm_core::Command::RotateTag,
            BaseCommand::IncreaseMainWidth => leftwm_core::Command::IncreaseMainWidth(
                i8::from_str(&self.value).context("invalid width value for IncreaseMainWidth")?,
            ),
            BaseCommand::DecreaseMainWidth => leftwm_core::Command::DecreaseMainWidth(
                i8::from_str(&self.value).context("invalid width value for DecreaseMainWidth")?,
            ),
            BaseCommand::SetMarginMultiplier => leftwm_core::Command::SetMarginMultiplier(
                f32::from_str(&self.value)
                    .context("invalid margin multiplier for SetMarginMultiplier")?,
            ),
            BaseCommand::UnloadTheme => leftwm_core::Command::Other("UnloadTheme".into()),
            BaseCommand::LoadTheme => leftwm_core::Command::Other(format!(
                "LoadTheme {}",
                ensure_non_empty!(self.value.clone())
            )),
        };

        Ok(leftwm_core::Keybind {
            command,
            modifier: self
                .modifier
                .as_ref()
                .unwrap_or(&"None".into())
                .clone()
                .into(),
            key: self.key.clone(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
#[serde(untagged)]
pub enum Modifier {
    Single(String),
    List(Vec<String>),
}

impl Modifier {
    pub fn is_empty(&self) -> bool {
        match self {
            Modifier::Single(single) => single.is_empty(),
            Modifier::List(list) => list.is_empty(),
        }
    }
}

impl std::convert::From<Modifier> for Vec<String> {
    fn from(m: Modifier) -> Self {
        match m {
            Modifier::Single(modifier) => vec![modifier],
            Modifier::List(modifiers) => modifiers,
        }
    }
}

impl IntoIterator for &Modifier {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let ms = match self {
            Modifier::Single(m) => vec![m.clone()],
            Modifier::List(ms) => ms.clone(),
        };
        ms.into_iter()
    }
}

impl std::convert::From<Vec<String>> for Modifier {
    fn from(l: Vec<String>) -> Self {
        Self::List(l)
    }
}

impl std::convert::From<&str> for Modifier {
    fn from(m: &str) -> Self {
        Self::Single(m.to_owned())
    }
}

impl std::fmt::Display for Modifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(modifier) => write!(f, "{}", modifier),
            Self::List(modifiers) => write!(f, "{}", modifiers.join("+")),
        }
    }
}

impl Modifier {
    pub fn sort_unstable(&mut self) {
        match self {
            Self::Single(_) => {}
            Self::List(modifiers) => modifiers.sort_unstable(),
        }
    }
}
