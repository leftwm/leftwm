use crate::config::Config;
use crate::display_servers::DisplayServer;
use crate::state::State;
use crate::utils::child_process::Children;
use std::sync::{atomic::AtomicBool, Arc};

/// Maintains current program state.
#[derive(Debug)]
pub struct Manager<C, SERVER> {
    pub state: State<C>,

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
        let display_server = SERVER::new(&config);

        Self {
            state: State::new(config),
            children: Default::default(),
            reap_requested: Default::default(),
            reload_requested: false,
            display_server,
        }
    }

    pub fn register_child_hook(&self) {
        crate::child_process::register_child_hook(self.reap_requested.clone());
    }

    /// Soft reload the worker without saving state.
    pub fn hard_reload(&mut self) {
        self.reload_requested = true;
    }
}

#[cfg(test)]
impl Manager<crate::config::TestConfig, crate::display_servers::MockDisplayServer> {
    pub fn new_test(tags: Vec<String>) -> Self {
        Self::new(crate::config::TestConfig { tags })
    }
}
