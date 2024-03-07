#[cfg(test)]
use leftwm_layouts::layouts::Layouts;

use crate::config::Config;
use crate::display_servers::DisplayServer;
use crate::state::State;
use crate::utils::child_process::Children;
use std::sync::{atomic::AtomicBool, Arc};

/// Maintains current program state.
#[derive(Debug)]
pub struct Manager<C, SERVER> {
    pub state: State,
    pub config: C,

    pub(crate) children: Children,
    pub(crate) reap_requested: Arc<AtomicBool>,
    pub(crate) reload_requested: bool,
    pub display_server: SERVER,
}

impl<C, SERVER> Manager<C, SERVER>
where
    C: Config,
    SERVER: DisplayServer,
{
    pub fn new(config: C) -> Self {
        Self {
            display_server: SERVER::new(&config),
            state: State::new(&config),
            config,
            children: Default::default(),
            reap_requested: Default::default(),
            reload_requested: false,
        }
    }
}

impl<C, SERVER> Manager<C, SERVER> {
    pub fn register_child_hook(&self) {
        crate::child_process::register_child_hook(self.reap_requested.clone());
    }

    /// Soft reload the worker without saving state.
    pub fn hard_reload(&mut self) {
        self.reload_requested = true;
    }
}

impl<C: Config, SERVER: DisplayServer> Manager<C, SERVER> {
    /// Reload the configuration of the running [`Manager`].
    pub fn load_theme_config(&mut self) -> bool {
        let focused = self
            .state
            .focus_manager
            .window_history
            .front()
            .and_then(|o| *o);
        self.display_server
            .reload_config(&self.config, focused, &self.state.windows);
        self.state.load_theme_config(&self.config);
        true
    }
}

#[cfg(test)]
impl Manager<crate::config::tests::TestConfig, crate::display_servers::MockDisplayServer> {
    pub fn new_test(tags: Vec<String>) -> Self {
        use crate::config::tests::TestConfig;
        let defs = Layouts::default().layouts;
        let names = defs.iter().map(|def| def.name.clone()).collect();
        Self::new(TestConfig {
            tags,
            layouts: names,
            layout_definitions: defs,
            ..TestConfig::default()
        })
    }

    pub fn new_test_with_border(tags: Vec<String>, border_width: i32) -> Self {
        use crate::config::tests::TestConfig;
        let defs = Layouts::default().layouts;
        let names = defs.iter().map(|def| def.name.clone()).collect();
        Self::new(TestConfig {
            tags,
            layouts: names,
            layout_definitions: defs,
            border_width,
            single_window_border: false,
            ..TestConfig::default()
        })
    }

    pub fn new_test_with_outputs(outputs: Vec<String>) -> Self {
        use crate::config::tests::TestConfig;
        use crate::config::Workspace;
        Self::new(TestConfig {
            workspaces: Some(
                outputs
                    .into_iter()
                    .map(|output| Workspace {
                        output,
                        ..Default::default()
                    })
                    .collect(),
            ),
            ..TestConfig::default()
        })
    }
}
