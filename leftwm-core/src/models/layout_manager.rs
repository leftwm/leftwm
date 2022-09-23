use super::Tag;
use crate::{config::Config, layouts::Layout, Workspace};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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
    pub layouts_per_workspaces: HashMap<(String, usize), Vec<Layout>>,
}

impl LayoutManager {
    pub fn new(config: &impl Config) -> Self {
        let mut layouts_per_workspaces: HashMap<(String, usize), Vec<Layout>> = HashMap::default();
        config
            .workspaces()
            .unwrap_or_default()
            .iter()
            .for_each(|ws| {
                layouts_per_workspaces.insert(
                    (
                        ws.output.clone(),
                        layouts_per_workspaces
                            .keys()
                            .filter(|&key| key.0 == ws.output)
                            .count()
                            + 1,
                    ),
                    ws.layouts.clone().unwrap_or_default(),
                );
            });

        Self {
            mode: config.layout_mode(),
            layouts: config.layouts(),
            layouts_per_workspaces,
        }
    }

    pub fn new_layout(&self, output: &str, num: usize) -> Layout {
        *self
            .layouts(output, num)
            .first()
            .unwrap_or(&Layout::default())
    }

    pub fn next_layout(&self, workspace: &Workspace) -> Layout {
        let layouts = self.layouts(&workspace.output, workspace.num);

        let next = match layouts.iter().position(|&x| x == workspace.layout) {
            Some(index) if index == layouts.len() - 1 => layouts.first(),
            Some(index) => layouts.get(index + 1),
            None => None,
        };

        // If no layout was found, return the first in the list, in case of a
        // SoftReload with a new list that does not include the current layout.
        *next.unwrap_or_else(|| layouts.first().unwrap_or(&workspace.layout))
    }

    pub fn previous_layout(&self, workspace: &Workspace) -> Layout {
        let layouts = self.layouts(&workspace.output, workspace.num);

        let next = match layouts.iter().position(|&x| x == workspace.layout) {
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

    fn layouts(&self, output: &str, num: usize) -> &Vec<Layout> {
        self.layouts_per_workspaces
            .get(&(output.to_owned(), num))
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
                    layouts: Some(vec![
                        Layout::CenterMain,
                        Layout::CenterMainBalanced,
                        Layout::MainAndDeck,
                    ]),
                    output: String::from("TEST"),
                    ..Default::default()
                },
                crate::config::Workspace {
                    output: String::from("TEST"),
                    ..Default::default()
                },
                crate::config::Workspace {
                    layouts: Some(vec![]),
                    output: String::from("TEST"),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };

        LayoutManager::new(&config)
    }

    fn workspace(num: usize, layout: Layout) -> Workspace {
        Workspace::new(
            BBox {
                width: 0,
                height: 0,
                x: 0,
                y: 0,
            },
            layout,
            None,
            String::from("TEST"),
            num,
        )
    }

    #[test]
    fn layouts_should_fallback_to_the_global_list() {
        let layout_manager = layout_manager();

        assert_eq!(
            layout_manager.layouts(&String::from("TEST"), 2),
            &layout_manager.layouts
        ); // layouts = None
        assert_eq!(
            layout_manager.layouts(&String::from("TEST"), 3),
            &layout_manager.layouts
        ); // layouts = vec![]
        assert_eq!(
            layout_manager.layouts(&String::from("TEST"), 4),
            &layout_manager.layouts
        ); // Non existent num
        assert_eq!(
            layout_manager.layouts(&String::from("NONE"), 1),
            &layout_manager.layouts
        ); // Non existent output
    }

    #[test]
    fn next_layout_basic() {
        let layout_manager = layout_manager();
        let workspace = workspace(1, Layout::CenterMainBalanced);

        assert_eq!(layout_manager.next_layout(&workspace), Layout::MainAndDeck);
    }

    #[test]
    fn next_layout_should_cycle() {
        let layout_manager = layout_manager();
        let workspace = workspace(1, Layout::MainAndDeck);

        assert_eq!(layout_manager.next_layout(&workspace), Layout::CenterMain);
    }

    #[test]
    fn next_layout_fallback_to_global_layouts() {
        let layout_manager = layout_manager();

        let workspace = workspace(2, Layout::EvenVertical);
        assert_eq!(
            layout_manager.next_layout(&workspace),
            Layout::MainAndHorizontalStack
        );
    }

    #[test]
    fn next_layout_fallback_to_the_first_layout() {
        let layout_manager = layout_manager();
        let workspace = workspace(1, Layout::Fibonacci);

        assert_eq!(layout_manager.next_layout(&workspace), Layout::CenterMain);
    }

    #[test]
    fn prev_layout_basic() {
        let layout_manager = layout_manager();
        let workspace = workspace(1, Layout::CenterMainBalanced);

        assert_eq!(
            layout_manager.previous_layout(&workspace),
            Layout::CenterMain
        );
    }

    #[test]
    fn prev_layout_should_cycle() {
        let layout_manager = layout_manager();
        let workspace = workspace(1, Layout::CenterMain);

        assert_eq!(
            layout_manager.previous_layout(&workspace),
            Layout::MainAndDeck
        );
    }

    #[test]
    fn previous_layout_fallback_to_global_layouts() {
        let layout_manager = layout_manager();
        let workspace = workspace(3, Layout::EvenVertical);

        assert_eq!(layout_manager.previous_layout(&workspace), Layout::Monocle);
    }

    #[test]
    fn previous_layout_fallback_to_the_first_layout() {
        let layout_manager = layout_manager();
        let workspace = workspace(1, Layout::Fibonacci);

        assert_eq!(
            layout_manager.previous_layout(&workspace),
            Layout::CenterMain
        );
    }
}
