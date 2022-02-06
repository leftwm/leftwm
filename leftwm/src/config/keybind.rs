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

impl Keybind {
    pub fn try_convert_to_lefthk_keybind(
        &self,
        config: &Config,
    ) -> Result<lefthk_core::config::Keybind> {
        let value_is_some = !self.value.is_empty();
        match &self.command {
            BaseCommand::Execute | BaseCommand::ToggleScratchPad | BaseCommand::LoadTheme => {
                ensure!(value_is_some, "value must not be empty");
            }
            BaseCommand::GotoTag => {
                usize::from_str(&self.value).context("invalid index value for GotoTag")?;
            }
            BaseCommand::FocusWindowTop if value_is_some => {
                bool::from_str(&self.value).context("invalid boolean value for FocusWindowTop")?;
            }
            BaseCommand::MoveToTag => {
                usize::from_str(&self.value).context("invalid index value for SendWindowToTag")?;
            }
            BaseCommand::SetLayout => {
                Layout::from_str(&self.value)
                    .context("could not parse layout for command SetLayout")?;
            }
            BaseCommand::IncreaseMainWidth => {
                i8::from_str(&self.value).context("invalid width value for IncreaseMainWidth")?;
            }
            BaseCommand::DecreaseMainWidth => {
                i8::from_str(&self.value).context("invalid width value for DecreaseMainWidth")?;
            }
            BaseCommand::SetMarginMultiplier => {
                f32::from_str(&self.value)
                    .context("invalid margin multiplier for SetMarginMultiplier")?;
            }
            _ => {}
        }

        let command: String = if self.command == BaseCommand::Execute {
            self.value.clone()
        } else {
            let mut head = "leftwm-command ".to_owned();
            let mut command_parts: String = self.command.into();
            if !self.value.is_empty() {
                let args = match &self.command {
                    BaseCommand::GotoTag => {
                        format!(" {} {}", self.value, config.disable_current_tag_swap)
                    }
                    _ => format!(" {}", self.value),
                };
                command_parts.push_str(&args);
            }
            head.push_str(&format!("\"{}\"", command_parts));
            head
        };
        Ok(lefthk_core::config::Keybind {
            command: lefthk_core::config::Command::Execute(command),
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
