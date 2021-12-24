use super::Tag;
use crate::{config::Config, layouts::Layout, Workspace};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub layouts_per_workspaces: HashMap<i32, Vec<Layout>>,
}

impl LayoutManager {
    pub fn new(config: &impl Config) -> Self {
        let layouts_per_workspaces = config
            .workspaces()
            .unwrap_or_default()
            .iter()
            .filter_map(|w| w.id.map(|id| (id, w.layouts.clone())))
            .collect();

        Self {
            mode: config.layout_mode(),
            layouts: config.layouts(),
            layouts_per_workspaces,
        }
    }

    pub fn new_layout(&self) -> Layout {
        *self.layouts.first().unwrap_or(&Layout::default())
    }

    pub fn next_layout(&self, layout: Layout, workspace: &Workspace) -> Layout {
        *workspace
            .id
            .and_then(|id| self.layouts_per_workspaces.get(&id))
            .or(Some(&self.layouts))
            .and_then(|layouts| match layouts.iter().position(|&x| x == layout) {
                Some(index) if index == layouts.len() - 1 => layouts.first(),
                Some(index) => layouts.get(index + 1),
                None => None,
            })
            .unwrap_or_default()
    }

    pub fn previous_layout(&self, layout: Layout, workspace: &Workspace) -> Layout {
        *workspace
            .id
            .and_then(|id| self.layouts_per_workspaces.get(&id))
            .or(Some(&self.layouts))
            .and_then(|layouts| match layouts.iter().position(|&x| x == layout) {
                Some(index) if index == 0 => layouts.last(),
                Some(index) => layouts.get(index - 1),
                None => None,
            })
            .unwrap_or_default()
    }

    pub fn update_layouts(
        &self,
        workspaces: &mut Vec<Workspace>,
        mut tags: Vec<&mut Tag>,
    ) -> Option<bool> {
        for workspace in workspaces {
            let tag = tags.iter_mut().find(|t| t.id == workspace.tags[0])?;
            match self.mode {
                LayoutMode::Workspace => {
                    tag.set_layout(workspace.layout, workspace.main_width_percentage);
                }
                LayoutMode::Tag => {
                    workspace.layout = tag.layout;
                    workspace.main_width_percentage = tag.main_width_percentage;
                }
            }
        }
        Some(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::TestConfig;
    use crate::models::BBox;

    use super::*;

    fn layout_manager() -> LayoutManager {
        let config = TestConfig {
            layouts: vec![
                Layout::Monocle,
                Layout::EvenVertical,
                Layout::MainAndHorizontalStack,
            ],
            workspaces: Some(vec![crate::config::Workspace {
                id: Some(0),
                layouts: vec![
                    Layout::CenterMain,
                    Layout::CenterMainBalanced,
                    Layout::MainAndDeck,
                ],
                ..Default::default()
            }]),
            ..Default::default()
        };

        LayoutManager::new(&config)
    }

    fn workspace(id: i32, layout: Layout) -> Workspace {
        Workspace::new(
            Some(id),
            BBox {
                width: 0,
                height: 0,
                x: 0,
                y: 0,
            },
            layout,
            None,
        )
    }

    #[test]
    fn next_layout_basic() {
        let layout_manager = layout_manager();
        let workspace = workspace(0, Layout::CenterMainBalanced);

        assert_eq!(
            layout_manager.next_layout(workspace.layout, &workspace),
            Layout::MainAndDeck
        );
    }

    #[test]
    fn next_layout_should_cycle() {
        let layout_manager = layout_manager();
        let workspace = workspace(0, Layout::MainAndDeck);

        assert_eq!(
            layout_manager.next_layout(workspace.layout, &workspace),
            Layout::CenterMain
        );
    }

    #[test]
    fn next_layout_fallback_to_global_layouts() {
        let layout_manager = layout_manager();
        let workspace = workspace(1, Layout::EvenVertical);

        assert_eq!(
            layout_manager.next_layout(workspace.layout, &workspace),
            Layout::MainAndHorizontalStack
        );
    }

    #[test]
    fn prev_layout_basic() {
        let layout_manager = layout_manager();
        let workspace = workspace(0, Layout::CenterMainBalanced);

        assert_eq!(
            layout_manager.previous_layout(workspace.layout, &workspace),
            Layout::CenterMain
        );
    }

    #[test]
    fn prev_layout_should_cycle() {
        let layout_manager = layout_manager();
        let workspace = workspace(0, Layout::CenterMain);

        assert_eq!(
            layout_manager.previous_layout(workspace.layout, &workspace),
            Layout::MainAndDeck
        );
    }

    #[test]
    fn previous_layout_fallback_to_global_layouts() {
        let layout_manager = layout_manager();
        let workspace = workspace(1, Layout::EvenVertical);

        assert_eq!(
            layout_manager.previous_layout(workspace.layout, &workspace),
            Layout::Monocle
        );
    }
}
