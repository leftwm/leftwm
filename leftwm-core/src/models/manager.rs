#[cfg(test)]
use leftwm_layouts::layouts::Layouts;

use crate::config::Config;
use crate::display_servers::DisplayServer;
use crate::state::State;
use crate::utils::child_process::Children;
use std::sync::{atomic::AtomicBool, Arc};

use super::Handle;

/// Maintains current program state.
#[derive(Debug)]
pub struct Manager<H: Handle, C, SERVER> {
    pub state: State<H>,
    pub config: C,

    pub(crate) children: Children,
    pub(crate) reap_requested: Arc<AtomicBool>,
    pub(crate) reload_requested: bool,
    pub display_server: SERVER,
}

impl<H: Handle, C, SERVER> Manager<H, C, SERVER>
where
    C: Config,
    SERVER: DisplayServer<H>,
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

impl<H: Handle, C, SERVER> Manager<H, C, SERVER> {
    pub fn register_child_hook(&self) {
        crate::child_process::register_child_hook(self.reap_requested.clone());
    }

    /// Soft reload the worker without saving state.
    pub fn hard_reload(&mut self) {
        self.reload_requested = true;
    }
}

impl<H: Handle, C: Config, SERVER: DisplayServer<H>> Manager<H, C, SERVER> {
    /// Reload the configuration of the running [`Manager`].
    pub fn reload_config(&mut self) -> bool {
        let focused = self.state.focus_manager.window_history.front();
        self.display_server
            .load_config(&self.config, focused, &self.state.windows);
        self.state.load_config(&self.config);
        true
    }
}

#[cfg(test)]
impl
    Manager<
        crate::models::window::MockHandle,
        crate::config::tests::TestConfig,
        crate::display_servers::MockDisplayServer<crate::models::window::MockHandle>,
    >
{
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
}
