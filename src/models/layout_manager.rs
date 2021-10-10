use crate::layouts::Layout;
use crate::{config::Config, Workspace};

use serde::{Deserialize, Serialize};

use super::{manager, workspace, Tag};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum LayoutMode {
    Tag,
    Workspace,
}

impl Default for LayoutMode {
    fn default() -> Self {
        Self::Workspace
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LayoutManager {
    pub mode: LayoutMode,
    pub layouts: Vec<Layout>,
}

impl LayoutManager {
    pub fn new(config: &impl Config) -> Self {
        Self {
            mode: config.layout_mode(),
            layouts: config.layouts(),
        }
    }

    pub fn new_layout(&self) -> Layout {
        *self.layouts.first().unwrap_or(&Layout::default())
    }

    pub fn next_layout(&self, layout: Layout) -> Layout {
        let mut index = match self.layouts.iter().position(|&x| x == layout) {
            Some(x) => x as isize,
            None => return Layout::default(),
        } + 1;
        if index >= self.layouts.len() as isize {
            index = 0;
        }
        self.layouts[index as usize]
    }

    pub fn previous_layout(&self, layout: Layout) -> Layout {
        let mut index = match self.layouts.iter().position(|&x| x == layout) {
            Some(x) => x as isize,
            None => return Layout::default(),
        } - 1;
        if index < 0 {
            index = self.layouts.len() as isize - 1;
        }
        self.layouts[index as usize]
    }

    pub fn update_layouts(
        &self,
        workspaces: &mut Vec<Workspace>,
        tags: &mut Vec<Tag>,
    ) -> Option<bool> {
        for workspace in workspaces {
            let tag = tags.iter_mut().find(|t| t.id == workspace.tags[0])?;
            match self.mode {
                LayoutMode::Workspace => tag.set_layout(workspace.layout),
                LayoutMode::Tag => workspace.layout = tag.layout,
            }
        }
        Some(true)
    }
}
