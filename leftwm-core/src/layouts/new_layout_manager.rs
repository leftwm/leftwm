use crate::{config::Config, utils::helpers::cycle_vec};
use leftwm_layouts::LayoutDefinition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::LayoutMode;

/// The [`LayoutManager`] holds the actual [`LayoutDefinitions`],
/// All references to "layouts" on Workspace or Tag are just
/// the layout name(s) as String pointing to the value
/// stored here
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewLayoutManager {
    /// LayoutMode to be used when applying layouts
    mode: LayoutMode,

    /// All the available layout definitions. Loaded from the config and
    /// to be unchanged during runtime. The layout manager shall make
    /// copies of those definitions for the specific workspaces and tags.
    available_definitions: Vec<LayoutDefinition>,

    /// The actual, modifiable layout definitions grouped by either
    /// Workspace or Tag, depending on the configured [`LayoutMode`].
    layouts: HashMap<usize, Vec<LayoutDefinition>>,
}

impl NewLayoutManager {
    /// Create a new [`LayoutManager`] from the config
    pub fn new(config: &impl Config) -> Self {
        let available_definitions: Vec<LayoutDefinition> = config
            .layout_definitions()
            .iter()
            .filter(|def| config.layouts().contains(&def.name))
            .map(std::clone::Clone::clone)
            .collect();

        // TODO: implement the workspace -> layouts config (available layouts may differ per workspace)
        //config.workspaces().unwrap().iter().for_each(|ws| ws.layouts)

        Self {
            mode: config.layout_mode(),
            available_definitions,
            layouts: HashMap::new(),
        }
    }

    /// Get back either the workspace ID or the tag ID, based on the current [`LayoutMode`]
    fn id(&self, wsid: usize, tagid: usize) -> usize {
        match self.mode {
            LayoutMode::Tag => tagid,
            LayoutMode::Workspace => wsid,
        }
    }

    fn layouts(&mut self, wsid: usize, tagid: usize) -> &Vec<LayoutDefinition> {
        let id = self.id(wsid, tagid);
        self.layouts
            .entry(id)
            .or_insert_with(|| self.available_definitions.clone())
    }

    fn layouts_mut(&mut self, wsid: usize, tagid: usize) -> &mut Vec<LayoutDefinition> {
        let id = self.id(wsid, tagid);
        self.layouts
            .entry(id)
            .or_insert_with(|| self.available_definitions.clone())
    }

    /// Get the current [`LayoutDefinition`] for the provided workspace / tag context
    pub fn layout(&mut self, wsid: usize, tagid: usize) -> &LayoutDefinition {
        // TODO: prevent panic
        self.layouts(wsid, tagid).first().unwrap()
    }

    /// Get the current [`LayoutDefinition`] for the provided workspace / tag context as mutable
    pub fn layout_mut(&mut self, wsid: usize, tagid: usize) -> &mut LayoutDefinition {
        // TODO: prevent panic
        self.layouts_mut(wsid, tagid).first_mut().unwrap()
    }

    pub fn cycle_next_layout(&mut self, wsid: usize, tagid: usize) {
        cycle_vec(self.layouts_mut(wsid, tagid), 1);
    }

    pub fn cycle_previous_layout(&mut self, wsid: usize, tagid: usize) {
        cycle_vec(self.layouts_mut(wsid, tagid), -1);
    }

    pub fn set_layout(&mut self, wsid: usize, tagid: usize, name: &str) {
        let i = self
            .layouts(wsid, tagid)
            .iter()
            .enumerate()
            .find(|(_, layout)| layout.name == name)
            .map(|(i, _)| i);

        match i {
            Some(index) => cycle_vec(self.layouts_mut(wsid, tagid), -(index as i32)),
            None => None,
        };
    }

    // todo: reset fn, that resets all the layout-definitions to their unchanged properties

    /*fn layouts(&self, workspace_id: Option<i32>, tag_id: usize) -> &Vec<LayoutDefinition> {
        match self.mode {
            LayoutMode::Tag => self
                .layouts_per_tags
                .get(&tag_id)
                .unwrap_or(&self.all_definitions),
            LayoutMode::Workspace => workspace_id
                .and_then(|wsid| self.layouts_per_workspaces.get(&wsid))
                .unwrap_or(&self.all_definitions),
        }
    }

    fn layout(&self, workspace_id: Option<i32>, tag_id: usize, name: String) -> &LayoutDefinition {
        self.layouts(workspace_id, tag_id)
            .iter()
            .find(|def| def.name.eq(&name))
            .unwrap()
    }*/

    /*pub fn layout(&self, workspace_id: Option<i32>, tag_id: usize) -> &LayoutDefinition {
        match self.mode {
            LayoutMode::Tag => self.,
            LayoutMode::Workspace => todo!(),
        }
    }*/

    /*pub fn new_layout(&self, workspace_id: Option<i32>) -> String {
        self.layouts(workspace_id)
            .first()
            .unwrap_or(&LayoutDefinition::default())
            .name
            .clone()
    }

    pub fn next_layout(&self, workspace: &Workspace) -> String {
        crate::utils::helpers::relative_find(
            self.layouts(workspace.id),
            |o| o.name == workspace.layout,
            1,
            true,
        )
        .map(|d| d.name.to_owned())
        .unwrap_or_else(|| LayoutDefinition::default().name)
    }

    pub fn previous_layout(&self, workspace: &Workspace) -> String {
        crate::utils::helpers::relative_find(
            self.layouts(workspace.id),
            |o| o.name == workspace.layout,
            -1,
            true,
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
    }*/

    /*pub fn apply(&self, name: &String, windows: &Vec<&mut Window>, ws: &Workspace) {
        let def = self
            .all_definitions
            .iter()
            .find(|x| x.name == *name)
            .unwrap_or_default();

        let container = Rect {
            x: ws.x(),
            y: ws.y(),
            h: ws.height().unsigned_abs(),
            w: ws.width().unsigned_abs(),
        };

        let rects = leftwm_layouts::apply(def, windows.len(), &container);
    }*/
}

#[cfg(test)]
mod tests {
    use leftwm_layouts::Layouts;

    use crate::{
        config::tests::TestConfig,
        layouts::{self, EVEN_VERTICAL, MONOCLE},
    };

    use super::NewLayoutManager;

    fn layout_manager() -> NewLayoutManager {
        let config = TestConfig {
            layouts: vec![
                layouts::MONOCLE.to_string(),
                layouts::EVEN_VERTICAL.to_string(),
                layouts::MAIN_AND_HORIZONTAL_STACK.to_string(),
            ],
            layout_definitions: Layouts::default().layouts,
            workspaces: Some(vec![
                crate::config::Workspace {
                    layouts: Some(vec![
                        layouts::CENTER_MAIN.to_string(),
                        layouts::CENTER_MAIN_BALANCED.to_string(),
                        layouts::MAIN_AND_DECK.to_string(),
                    ]),
                    ..Default::default()
                },
                crate::config::Workspace {
                    ..Default::default()
                },
                crate::config::Workspace {
                    layouts: Some(vec![]),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };

        NewLayoutManager::new(&config)
    }

    #[test]
    fn layouts_should_fallback_to_the_global_list() {
        let layout_manager = layout_manager();
        assert_eq!(1, layout_manager.id(1, 2));
    }

    #[test]
    fn monocle_layout_only_has_single_windows() {
        let mut layout_manager = layout_manager();
        layout_manager.set_layout(1, 1, MONOCLE);
        assert_eq!(MONOCLE, &layout_manager.layout(1, 1).name);
        layout_manager.set_layout(1, 1, EVEN_VERTICAL);
        assert_eq!(EVEN_VERTICAL, &layout_manager.layout(1, 1).name);
    }
}
