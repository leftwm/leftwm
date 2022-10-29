use super::Tag;
use crate::{config::Config, layouts::Layout, Workspace};
use leftwm_layouts::LayoutDefinition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Tag,
    Workspace,
}

impl Default for LayoutMode {
    fn default() -> Self {
        Self::Tag // todo: update wiki with new default value
    }
}

/// The LayoutManager holds the actual LayoutDefinitions,
/// All references to "layouts" on Workspace or Tag are just
/// the layout name(s) as String pointing to the value
/// stored here
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LayoutManager {
    pub mode: LayoutMode,
    pub layouts: Vec<LayoutDefinition>,
    pub layouts_per_workspaces: HashMap<i32, Vec<LayoutDefinition>>,
}

impl LayoutManager {
    pub fn new(config: &impl Config) -> Self {
        let layouts_per_workspaces = config
            .workspaces()
            .unwrap_or_default()
            .iter()
            .map(|ws| {
                (
                    ws.id.unwrap_or_default(),
                    ws.layouts.unwrap_or_default().iter()
                        .filter_map(|layout_name| config.layout_definitions().iter().find(|l| l.name == *layout_name))
                        .map(|definition| definition.clone())
                        .collect(),
                )
            })
            .collect();

        Self {
            mode: config.layout_mode(),
            layouts: config.layout_definitions().clone(),
            layouts_per_workspaces,
        }
    }

    pub fn new_layout(&self, workspace_id: Option<i32>) -> LayoutDefinition {
        *self
            .layouts(workspace_id)
            .first()
            .unwrap_or(&LayoutDefinition::default())
    }

    pub fn next_layout(&self, workspace: &Workspace) -> LayoutDefinition {
        let layouts = self.layouts(workspace.id);

        let next = match layouts.iter().position(|&x| x.name == workspace.layout) {
            Some(index) if index == layouts.len() - 1 => layouts.first(),
            Some(index) => layouts.get(index + 1),
            None => None,
        };

        // If no layout was found, return the first in the list, in case of a
        // SoftReload with a new list that does not include the current layout.
        *next.unwrap_or_else(|| layouts.first().unwrap_or(&workspace.layout))
    }

    pub fn previous_layout(&self, workspace: &Workspace) -> Layout {
        let layouts = self.layouts(workspace.id);

        let next = match layouts.iter().position(|&x| x.name == workspace.layout) {
            Some(index) if index == 0 => layouts.last(),
            Some(index) => layouts.get(index - 1),
            None => None,
        };

        // If no layout was found, return the first in the list, in case of a
        // SoftReload with a new list that does not include the current layout.
        *next.unwrap_or_else(|| layouts.first().unwrap_or(&workspace.layout))
    }

    pub fn update_layouts(
        &self,
        workspaces: &mut Vec<Workspace>,
        mut tags: Vec<&mut Tag>,
    ) -> Option<bool> {
        for workspace in workspaces {
            let tag = tags.iter_mut().find(|t| Some(t.id) == workspace.tag)?;
            match self.mode {
                LayoutMode::Workspace => {
                    tag.set_layout(workspace.layout);
                }
                LayoutMode::Tag => {
                    workspace.layout = tag.layout;
                }
            }
        }
        Some(true)
    }

    fn layouts(&self, workspace_id: Option<i32>) -> &Vec<LayoutDefinition> {
        workspace_id
            .and_then(|id| self.layouts_per_workspaces.get(&id))
            .and_then(|layouts| {
                if layouts.is_empty() {
                    None
                } else {
                    Some(layouts)
                }
            })
            .unwrap_or(&self.layouts)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::tests::TestConfig;
    use crate::models::BBox;

    use super::*;

    fn layout_manager() -> LayoutManager {
        let config = TestConfig {
            layouts: vec![
                Layout::Monocle,
                Layout::EvenVertical,
                Layout::MainAndHorizontalStack,
            ],
            workspaces: Some(vec![
                crate::config::Workspace {
                    id: Some(0),
                    layouts: Some(vec![
                        String::from("CenterMain"),
                        String::from("CenterMainBalanced"),
                        String::from("MainAndDeck"),
                    ]),
                    ..Default::default()
                },
                crate::config::Workspace {
                    id: Some(1),
                    ..Default::default()
                },
                crate::config::Workspace {
                    id: Some(2),
                    layouts: Some(vec![]),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };

        LayoutManager::new(&config)
    }

    fn workspace(id: i32, layout: String) -> Workspace {
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
    fn layouts_should_fallback_to_the_global_list() {
        let layout_manager = layout_manager();

        assert_eq!(layout_manager.layouts(Some(1)), &layout_manager.layouts); // layouts = None
        assert_eq!(layout_manager.layouts(Some(2)), &layout_manager.layouts); // layouts = vec[]!
        assert_eq!(layout_manager.layouts(Some(3)), &layout_manager.layouts); // Non existent id
        assert_eq!(layout_manager.layouts(None), &layout_manager.layouts);
    }

    #[test]
    fn next_layout_basic() {
        let layout_manager = layout_manager();
        let workspace = workspace(0, String::from("CenterMainBalanced"));
        assert_eq!(layout_manager.next_layout(&workspace).name, String::from("MainAndDeck"));
    }

    #[test]
    fn next_layout_should_cycle() {
        let layout_manager = layout_manager();
        let workspace = workspace(0, String::from("MainAndDeck"));
        assert_eq!(layout_manager.next_layout(&workspace).name, String::from("CenterMain"));
    }

    #[test]
    fn next_layout_fallback_to_global_layouts() {
        let layout_manager = layout_manager();

        let workspace = workspace(1, String::from("EvenVertical"));
        assert_eq!(
            layout_manager.next_layout(&workspace).name,
            String::from("MainAndHorizontalStack")
        );
    }

    #[test]
    fn next_layout_fallback_to_the_first_layout() {
        let layout_manager = layout_manager();
        let workspace = workspace(0, Layout::Fibonacci);

        assert_eq!(layout_manager.next_layout(&workspace), Layout::CenterMain);
    }

    #[test]
    fn prev_layout_basic() {
        let layout_manager = layout_manager();
        let workspace = workspace(0, Layout::CenterMainBalanced);

        assert_eq!(
            layout_manager.previous_layout(&workspace),
            Layout::CenterMain
        );
    }

    #[test]
    fn prev_layout_should_cycle() {
        let layout_manager = layout_manager();
        let workspace = workspace(0, Layout::CenterMain);

        assert_eq!(
            layout_manager.previous_layout(&workspace),
            Layout::MainAndDeck
        );
    }

    #[test]
    fn previous_layout_fallback_to_global_layouts() {
        let layout_manager = layout_manager();
        let workspace = workspace(2, Layout::EvenVertical);

        assert_eq!(layout_manager.previous_layout(&workspace), Layout::Monocle);
    }

    #[test]
    fn previous_layout_fallback_to_the_first_layout() {
        let layout_manager = layout_manager();
        let workspace = workspace(0, Layout::Fibonacci);

        assert_eq!(
            layout_manager.previous_layout(&workspace),
            Layout::CenterMain
        );
    }
}
