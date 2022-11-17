use super::Tag;
use crate::{config::Config, Workspace, Window};
use leftwm_layouts::{LayoutDefinition, geometry::Rect};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, borrow::Borrow};

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

    /// Get a copy of all the layout definitions in `defs` whose names
    /// are contained in `names`.
    fn map_name_to_definition(defs: &Vec<LayoutDefinition>, names: &Vec<String>) -> Vec<LayoutDefinition> {
        return names.iter()
            .filter_map(|name| defs.iter().find(|def| def.name == *name))
            .map(|def| def.to_owned())
            .collect();
    }

    /// Create a new `LayoutManager` from the config
    pub fn new(config: &impl Config) -> Self {
        let definitions = &config.layout_definitions();
        let layouts: Vec<LayoutDefinition> = Self::map_name_to_definition(definitions, &config.layouts());
        let layouts_per_workspaces: HashMap<i32, Vec<LayoutDefinition>> = config.workspaces().unwrap_or_default().iter()
            .map(|ws| {
                (
                    ws.id.unwrap_or_default(), // TODO: why is ws.id an Option?
                    Self::map_name_to_definition(definitions, ws.layouts.as_ref().unwrap_or(&Vec::new()).borrow()),
                )
            })
            .collect();
        Self {
            mode: config.layout_mode(),
            layouts,
            layouts_per_workspaces,
        }
    }

    pub fn new_layout(&self, workspace_id: Option<i32>) -> String {
        self
            .layouts(workspace_id)
            .first()
            .unwrap_or(&LayoutDefinition::default()).name.clone()
    }

    pub fn next_layout(&self, workspace: &Workspace) -> String {
        crate::utils::helpers::relative_find(
            self.layouts(workspace.id), 
            |o| o.name == workspace.layout,
             1,
              true
            )
            .map(|d| d.name.to_owned())
            .unwrap_or_else(|| LayoutDefinition::default().name)
    }

    pub fn previous_layout(&self, workspace: &Workspace) -> String {
        crate::utils::helpers::relative_find(
            self.layouts(workspace.id), 
            |o| o.name == workspace.layout,
             -1,
              true
            )
            .map(|d| d.name.to_owned())
            .unwrap_or_else(|| LayoutDefinition::default().name.to_owned())
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
                    tag.set_layout(workspace.layout.to_owned());
                }
                LayoutMode::Tag => {
                    workspace.layout = tag.layout.to_owned();
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

    pub fn apply(&self, name: &String, windows: &Vec<&mut Window>, ws: &Workspace) {
        let def = self.layouts.iter()
            .find(|x| x.name == *name)
            .unwrap_or(&LayoutDefinition::default());
        
        let container = Rect{
            x: ws.x(),
            y: ws.y(),
            h: ws.height().unsigned_abs(),
            w: ws.width().unsigned_abs(),
        };

        //let rects = leftwm_layouts::apply(def, windows.len(), container);
    }
}

#[cfg(test)]
mod tests {
    use crate::config::tests::TestConfig;
    use crate::layouts;
    use crate::models::BBox;

    use super::*;

    fn layout_manager() -> LayoutManager {
        let config = TestConfig {
            layouts: vec![
                layouts::MONOCLE.to_string(),
                layouts::EVEN_VERTICAL.to_string(),
                layouts::MAIN_AND_HORIZONTAL_STACK.to_string(),
            ],
            workspaces: Some(vec![
                crate::config::Workspace {
                    id: Some(0),
                    layouts: Some(vec![
                        layouts::CENTER_MAIN.to_string(),
                        layouts::CENTER_MAIN_BALANCED.to_string(),
                        layouts::MAIN_AND_DECK.to_string(),
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
        assert_eq!(layout_manager.next_layout(&workspace), String::from("MainAndDeck"));
    }

    #[test]
    fn next_layout_should_cycle() {
        let layout_manager = layout_manager();
        let workspace = workspace(0, String::from("MainAndDeck"));
        assert_eq!(layout_manager.next_layout(&workspace), String::from("CenterMain"));
    }

    #[test]
    fn next_layout_fallback_to_global_layouts() {
        let layout_manager = layout_manager();

        let workspace = workspace(1, String::from("EvenVertical"));
        assert_eq!(
            layout_manager.next_layout(&workspace),
            String::from("MainAndHorizontalStack")
        );
    }

    #[test]
    fn next_layout_fallback_to_the_first_layout() {
        let layout_manager = layout_manager();
        let workspace = workspace(0, String::from("Fibonacci"));

        assert_eq!(layout_manager.next_layout(&workspace), String::from("CenterMain"));
    }

    #[test]
    fn prev_layout_basic() {
        let layout_manager = layout_manager();
        let workspace = workspace(0, String::from("CenterMainBalanced"));

        assert_eq!(
            layout_manager.previous_layout(&workspace),
            String::from("CenterMain")
        );
    }

    #[test]
    fn prev_layout_should_cycle() {
        let layout_manager = layout_manager();
        let workspace = workspace(0, String::from("CenterMain"));

        assert_eq!(
            layout_manager.previous_layout(&workspace),
            String::from("MainAndDeck")
        );
    }

    #[test]
    fn previous_layout_fallback_to_global_layouts() {
        let layout_manager = layout_manager();
        let workspace = workspace(2, String::from("EvenVertical"));

        assert_eq!(layout_manager.previous_layout(&workspace), String::from("Monocle"));
    }

    #[test]
    fn previous_layout_fallback_to_the_first_layout() {
        let layout_manager = layout_manager();
        let workspace = workspace(0, String::from("Fibonacci"));

        assert_eq!(
            layout_manager.previous_layout(&workspace),
            String::from("CenterMain")
        );
    }
}
