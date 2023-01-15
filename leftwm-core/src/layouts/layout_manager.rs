use crate::{config::Config, utils::helpers::cycle_vec};
use leftwm_layouts::Layout;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::LayoutMode;

/// The [`LayoutManager`] holds the actual set of [`Layout`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LayoutManager {
    /// LayoutMode to be used when applying layouts
    mode: LayoutMode,

    /// All the available layout definitions. Loaded from the config and
    /// to be unchanged during runtime. The layout manager shall make
    /// copies of those definitions for the specific workspaces and tags.
    available_definitions: Vec<Layout>,

    /// The actual, modifiable layout definitions grouped by either
    /// Workspace or Tag, depending on the configured [`LayoutMode`].
    layouts: HashMap<usize, Vec<Layout>>,
}

impl LayoutManager {
    /// Create a new [`LayoutManager`] from the config
    pub fn new(config: &impl Config) -> Self {
        let mut available_definitions: Vec<Layout> = Vec::new();

        tracing::debug!(
            "Looking for layout definitions named: {:?}",
            config.layouts()
        );
        for name in config.layouts() {
            if let Some(def) = config
                .layout_definitions()
                .iter()
                .find(|def| def.name == name)
            {
                available_definitions.push(def.clone());
            } else {
                tracing::warn!("There is no Layout with the name {:?}", name);
            }
        }

        if available_definitions.is_empty() {
            tracing::warn!(
                "No Layouts were loaded from config - defaulting to a single default Layout"
            );
            available_definitions.push(Layout::default());
        }

        tracing::debug!(
            "The available layout definitions are: {:?}",
            available_definitions
        );

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

    fn layouts(&mut self, wsid: usize, tagid: usize) -> &Vec<Layout> {
        let id = self.id(wsid, tagid);
        self.layouts
            .entry(id)
            .or_insert_with(|| self.available_definitions.clone())
    }

    fn layouts_mut(&mut self, wsid: usize, tagid: usize) -> &mut Vec<Layout> {
        let id = self.id(wsid, tagid);
        self.layouts
            .entry(id)
            .or_insert_with(|| self.available_definitions.clone())
    }

    /// Get the current [`Layout`] for the provided workspace / tag context
    pub fn layout(&mut self, wsid: usize, tagid: usize) -> &Layout {
        let layouts = self.layouts(wsid, tagid);
        assert!(
            !layouts.is_empty(),
            "there should always be at least one layout definition"
        );
        self.layouts(wsid, tagid).first().unwrap()
    }

    /// Get the current [`Layout`] for the provided workspace / tag context as mutable
    pub fn layout_mut(&mut self, wsid: usize, tagid: usize) -> &mut Layout {
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
    use leftwm_layouts::layouts::Layouts;

    use crate::{
        config::tests::TestConfig,
        layouts::{self, EVEN_VERTICAL, MONOCLE},
    };

    use super::LayoutManager;

    fn layout_manager() -> LayoutManager {
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

        LayoutManager::new(&config)
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
