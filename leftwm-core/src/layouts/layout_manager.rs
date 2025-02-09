use crate::{config::Config, utils::helpers::cycle_vec};
use leftwm_layouts::Layout;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::LayoutMode;

/// The [`LayoutManager`] holds the actual set of [`Layout`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LayoutManager {
    /// `LayoutMode` to be used when applying layouts
    mode: LayoutMode,

    /// All the available layouts. Loaded from the config and
    /// to be unchanged during runtime. The layout manager shall make
    /// copies of those layouts for the specific workspaces and tags.
    available_layouts: Vec<Layout>,

    /// All the available layouts per workspace. Different workspaces may
    /// have different available layouts, if configured that way. If a
    /// workspace does not have its own set of available layouts, the
    /// global available layouts from [`available_layouts`] will be used instead.
    available_layouts_per_ws: HashMap<usize, Vec<Layout>>,

    /// The actual, modifiable layouts grouped by either
    /// Workspace or Tag, depending on the configured [`LayoutMode`].
    layouts: HashMap<usize, Vec<Layout>>,
}

impl LayoutManager {
    /// Create a new [`LayoutManager`] from the config
    pub fn new(config: &impl Config) -> Self {
        let mut available_layouts: Vec<Layout> = Vec::new();

        tracing::trace!("Looking for layouts named: {:?}", config.layouts());
        for name in config.layouts() {
            if let Some(def) = config
                .layout_definitions()
                .iter()
                .find(|def| def.name == name)
            {
                available_layouts.push(def.clone());
            } else {
                tracing::warn!("There is no Layout with the name {:?}", name);
            }
        }

        let mut available_layouts_per_ws: HashMap<usize, Vec<Layout>> = HashMap::new();

        for (i, ws) in config.workspaces().unwrap_or_default().iter().enumerate() {
            if let Some(ws_layout_names) = &ws.layouts {
                let wsid = i + 1;
                for ws_layout_name in ws_layout_names {
                    if let Some(layout) = config
                        .layout_definitions()
                        .iter()
                        .find(|layout| layout.name == *ws_layout_name)
                    {
                        available_layouts_per_ws
                            .entry(wsid)
                            .and_modify(|layouts| layouts.push(layout.clone()))
                            .or_insert_with(|| vec![layout.clone()]);
                    } else {
                        tracing::warn!("There is no Layout with the name {:?}, but was configured on workspace {:?}", ws_layout_name, wsid);
                    }
                }
            }
        }

        if available_layouts.is_empty() {
            tracing::warn!(
                "No Layouts were loaded from config - defaulting to a single default Layout"
            );
            available_layouts.push(Layout::default());
        }

        tracing::trace!("The general available layouts are: {:?}", available_layouts);
        tracing::trace!(
            "The workspace specific available layouts are: {:?}",
            available_layouts_per_ws
        );

        Self {
            mode: config.layout_mode(),
            available_layouts,
            available_layouts_per_ws,
            layouts: HashMap::new(),
        }
    }

    pub fn restore(&mut self, old: &LayoutManager) {
        if self.mode != old.mode {
            tracing::debug!("The LayoutMode has changed, layouts will not be restored");
            return;
        }
        // TODO we could eventually try to map available layouts as best as we can
        //      and only fallback to default for layouts not avialable anymore
        if self.available_layouts != old.available_layouts {
            tracing::debug!("The available Layouts have changed, layouts will not be restored");
            return;
        }
        if self.available_layouts_per_ws != old.available_layouts_per_ws {
            tracing::debug!(
                "The available Layouts per Workspace have changed, layouts will not be restored"
            );
            return;
        }
        self.layouts.clone_from(&old.layouts);
    }

    /// Get back either the workspace ID or the tag ID, based on the current [`LayoutMode`]
    fn id(&self, wsid: usize, tagid: usize) -> usize {
        match self.mode {
            LayoutMode::Tag => tagid,
            LayoutMode::Workspace => wsid,
        }
    }

    /// Get the layouts for the provided workspace / tag context
    ///
    /// If the layouts for the specific workspace / tag have not
    /// yet been set up, they will be initialized by copying
    /// from the [`Self::available_layouts`] or [`Self::available_layouts_per_ws`].
    fn layouts(&mut self, wsid: usize, tagid: usize) -> &Vec<Layout> {
        self.layouts_mut(wsid, tagid)
    }

    /// Get the mutable layouts for the provided workspace / tag context
    ///
    /// If the layouts for the specific workspace / tag have not
    /// yet been set up, they will be initialized by copying
    /// from the [`Self::available_layouts`] or [`Self::available_layouts_per_ws`].
    fn layouts_mut(&mut self, wsid: usize, tagid: usize) -> &mut Vec<Layout> {
        let id = self.id(wsid, tagid);
        self.layouts.entry(id).or_insert_with(|| match &self.mode {
            LayoutMode::Tag => self.available_layouts.clone(),
            LayoutMode::Workspace => self
                .available_layouts_per_ws
                .get(&wsid)
                .unwrap_or(&self.available_layouts)
                .clone(),
        })
    }

    /// Get the current [`Layout`] for the provided workspace / tag context
    ///
    /// This may return [`None`] if the layouts have not been set up for
    /// the specific tag / workspace. If unsure, it is probably wiser
    /// to use [`Self::layout(usize, usize)`], which will initialize
    /// the layouts automatically.
    pub fn layout_maybe(&self, wsid: usize, tagid: usize) -> Option<&Layout> {
        let id = self.id(wsid, tagid);
        self.layouts.get(&id).and_then(|vec| vec.first())
    }

    /// Get the current [`Layout`] for the provided workspace / tag context
    ///
    /// # Panics
    /// May panic if `available_layouts` is empty, which shouldn't happen because
    /// it always falls back to a default layout when it's empty
    pub fn layout(&mut self, wsid: usize, tagid: usize) -> &Layout {
        let layouts = self.layouts(wsid, tagid);
        assert!(
            !layouts.is_empty(),
            "there should be always at least one layout, because LeftWM must fallback to a default if empty"
        );
        layouts.first().unwrap()
    }

    /// Get the current [`Layout`] for the provided workspace / tag context as mutable
    ///
    /// # Panics
    /// May panic if `available_layouts` is empty, which shouldn't happen because
    /// it always falls back to a default layout when it's empty
    pub fn layout_mut(&mut self, wsid: usize, tagid: usize) -> &mut Layout {
        let layouts = self.layouts_mut(wsid, tagid);
        assert!(
            !layouts.is_empty(),
            "there should be always at least one layout, because LeftWM must fallback to a default if empty"
        );
        layouts.first_mut().unwrap()
    }

    pub fn cycle_next_layout(&mut self, wsid: usize, tagid: usize) {
        cycle_vec(self.layouts_mut(wsid, tagid), -1);
    }

    pub fn cycle_previous_layout(&mut self, wsid: usize, tagid: usize) {
        cycle_vec(self.layouts_mut(wsid, tagid), 1);
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

    // todo - low priority: reset fn, that resets all the layouts to their unchanged properties
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
        layout_manager.set_layout(2, 1, MONOCLE);
        assert_eq!(MONOCLE, &layout_manager.layout(2, 1).name);
        layout_manager.set_layout(2, 1, EVEN_VERTICAL);
        assert_eq!(EVEN_VERTICAL, &layout_manager.layout(2, 1).name);
    }
}
