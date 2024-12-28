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
impl
    Manager<
        crate::models::window::MockHandle,
        crate::config::tests::TestConfig,
        crate::display_servers::MockDisplayServer<crate::models::window::MockHandle>,
    >
{
    pub fn new_with_screens(
        config: crate::config::tests::TestConfig,
        screens: Vec<super::Screen<crate::models::window::MockHandle>>,
    ) -> Self {
        // needs to mimic what the display server would do when it starts,
        // specifically in respect to how workspaces are created for the screens
        // even more specifically, I'm recreating the behavior of `XlibDisplayServer::initial_events`)
        // by iterating through the configured workspaces and creating them for screens whose outputs match a workspace's output

        // figure out all the screens that need to be created
        let mut events = vec![];
        if let Some(workspaces) = config.workspaces() {
            for (i, wsc) in workspaces.iter().enumerate() {
                let mut screen = super::Screen::from(wsc);
                screen.root =
                    crate::models::WindowHandle(crate::models::window::MockHandle::default());
                // If there is a screen corresponding to the given output, create the workspace
                match screens.iter().find(|s| s.output == wsc.output) {
                    Some(output_match) => {
                        if wsc.relative.unwrap_or(false) {
                            screen.bbox.add(output_match.bbox);
                        }
                        screen.id = Some(i + 1);
                    }
                    None => continue,
                }

                // the only events we care about are ScreenCreate events, so instead of collecting DisplayEvents, we'll just collect Screens
                events.push(screen);
            }

            let auto_derive_workspaces: bool = config.auto_derive_workspaces() || events.is_empty();
            let mut next_id = workspaces.len() + 1;

            // If there is no hardcoded workspace layout, add every screen not mentioned in the config.
            if auto_derive_workspaces {
                screens
                    .iter()
                    .filter(|screen| !workspaces.iter().any(|wsc| wsc.output == screen.output))
                    .for_each(|screen| {
                        let mut s = screen.clone();
                        s.id = Some(next_id);
                        next_id += 1;
                        events.push(s);
                    });
            }
        }

        // now, create the manager
        let mut manager = Self::new(config);

        // and apply the events
        for screen in events {
            manager.screen_create_handler(screen);
        }

        manager
    }

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

#[cfg(test)]
mod pr_1301_issue {
    //! A set of tests to reproduce the issue described in the [comments of PR #1301](https://github.com/leftwm/leftwm/pull/1301#issuecomment-2542006937)
    //! where despite the default layout being set in the config, it was not being used,
    //! and instead "Grid" was being used for both workspaces since it was the first
    //! layout in the layouts list.

    use leftwm_layouts::layouts::Layouts;

    use crate::{
        config::tests::TestConfig,
        display_servers::MockDisplayServer,
        layouts,
        models::{BBox, MockHandle, Screen},
        Manager,
    };

    fn test_config() -> TestConfig {
        TestConfig {
            layouts: vec![
                layouts::FIBONACCI.to_string(),
                "Grid".to_string(),
                layouts::MONOCLE.to_string(),
            ],
            layout_definitions: Layouts::default().layouts,
            workspaces: Some(vec![
                crate::config::Workspace {
                    output: "DP-3".to_string(),
                    y: 0,
                    x: 0,
                    height: 1080,
                    width: 1920,
                    default_layout: Some(layouts::MONOCLE.to_string()),
                    ..Default::default()
                },
                crate::config::Workspace {
                    output: "DP-4".to_string(),
                    y: 1080,
                    x: 0,
                    height: 1080,
                    width: 1920,
                    default_layout: Some("Grid".to_string()),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        }
    }

    #[test]
    /// assume that the users screens are the same as the ones in the config
    ///
    /// this is to validate that the default layout is used when workspaces
    /// are configured properly
    fn default_layout_with_correct_screen() {
        let mut manager: Manager<MockHandle, TestConfig, MockDisplayServer<MockHandle>> =
            Manager::new_with_screens(
                test_config(),
                vec![
                    Screen::new(
                        BBox {
                            x: 0,
                            y: 0,
                            width: 1920,
                            height: 1080,
                        },
                        "DP-3".to_string(),
                    ),
                    Screen::new(
                        BBox {
                            x: 0,
                            y: 1080,
                            width: 1920,
                            height: 1080,
                        },
                        "DP-4".to_string(),
                    ),
                ],
            );

        assert_eq!(2, manager.state.workspaces.len());
        assert_eq!(
            layouts::MONOCLE,
            &manager.state.layout_manager.layout(1, 1).name
        );
        assert_eq!("Grid", &manager.state.layout_manager.layout(2, 1).name);
    }

    #[test]
    /// assume that the users screens are not the same as the ones in the config
    ///
    /// this is to reproduce the issue, and demonstrate that it is not necessarily a bug
    fn default_layout_with_incorrect_screen() {
        let mut manager: Manager<MockHandle, TestConfig, MockDisplayServer<MockHandle>> =
            Manager::new_with_screens(
                test_config(),
                vec![Screen::new(
                    BBox {
                        x: 0,
                        y: 0,
                        width: 1920,
                        height: 1080,
                    },
                    // notice how this is not one of the configured outputs
                    "eDP-1".to_string(),
                )],
            );

        // there is only one workspace
        assert_eq!(1, manager.state.workspaces.len());
        // that workspace is not one of the 2 configured
        assert_eq!(3, manager.state.workspaces[0].id);
        // so it defaults to the first layout in the list
        assert_eq!(
            layouts::FIBONACCI,
            &manager.state.layout_manager.layout(3, 1).name
        );
    }

    #[test]
    /// Assume that the user has 2 screens, but only one is configured
    fn default_layout_with_one_configured_screen() {
        let mut manager: Manager<MockHandle, TestConfig, MockDisplayServer<MockHandle>> =
            Manager::new_with_screens(
                test_config(),
                vec![
                    Screen::new(
                        BBox {
                            x: 0,
                            y: 0,
                            width: 1920,
                            height: 1080,
                        },
                        "DP-3".to_string(),
                    ),
                    Screen::new(
                        BBox {
                            x: 0,
                            y: 1080,
                            width: 1920,
                            height: 1080,
                        },
                        "eDP-1".to_string(),
                    ),
                ],
            );

        // there are 2 workspaces
        assert_eq!(2, manager.state.workspaces.len());
        // the first screen's workspace is configured as expected
        assert_eq!(
            layouts::MONOCLE,
            &manager.state.layout_manager.layout(1, 1).name
        );
        // and has the expected id
        assert_eq!(1, manager.state.workspaces[0].id);
        // the second screen has no configuration, and fallsback to defaults
        assert_eq!(
            layouts::FIBONACCI,
            &manager.state.layout_manager.layout(3, 1).name
        );
        // and has the expected id
        assert_eq!(3, manager.state.workspaces[1].id);
    }
}
