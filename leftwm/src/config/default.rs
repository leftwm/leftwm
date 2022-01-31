use super::{
    default_terminal, exit_strategy, BaseCommand, Config, Default, FocusBehaviour, Keybind,
    LayoutMode, ThemeSetting, LAYOUTS,
};

impl Default for Config {
    // We allow this because this function would be difficult to reduce. If someone would like to
    // move the commands builder out, perhaps make a macro, this function could be reduced in size
    // considerably.
    #[allow(clippy::too_many_lines)]
    fn default() -> Self {
        const WORKSPACES_NUM: usize = 10;
        let mut commands = vec![
            // Mod + p => Open dmenu
            Keybind {
                command: BaseCommand::Execute,
                value: "dmenu_run".to_owned(),
                modifier: Some(vec!["modkey".to_owned()].into()),
                key: "p".to_owned(),
            },
            // Mod + Shift + Enter => Open A Shell
            Keybind {
                command: BaseCommand::Execute,
                value: default_terminal().to_owned(),
                modifier: Some(vec!["modkey".to_owned(), "Shift".to_owned()].into()),
                key: "Return".to_owned(),
            },
            // Mod + Shift + q => kill focused window
            Keybind {
                command: BaseCommand::CloseWindow,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned(), "Shift".to_owned()].into()),
                key: "q".to_owned(),
            },
            // Mod + Shift + r => soft reload leftwm
            Keybind {
                command: BaseCommand::SoftReload,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned(), "Shift".to_owned()].into()),
                key: "r".to_owned(),
            },
            // Mod + Shift + x => exit leftwm
            Keybind {
                command: BaseCommand::Execute,
                value: exit_strategy().to_owned(),
                modifier: Some(vec!["modkey".to_owned(), "Shift".to_owned()].into()),
                key: "x".to_owned(),
            },
            // Mod + Ctrl + l => lock the screen
            Keybind {
                command: BaseCommand::Execute,
                value: "slock".to_owned(),
                modifier: Some(vec!["modkey".to_owned(), "Control".to_owned()].into()),
                key: "l".to_owned(),
            },
            // Mod + Shift + w => swap the tags on the last to active workspaces
            Keybind {
                command: BaseCommand::MoveToLastWorkspace,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned(), "Shift".to_owned()].into()),
                key: "w".to_owned(),
            },
            // Mod + w => move the active window to the previous workspace
            Keybind {
                command: BaseCommand::SwapTags,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned()].into()),
                key: "w".to_owned(),
            },
            Keybind {
                command: BaseCommand::MoveWindowUp,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned(), "Shift".to_owned()].into()),
                key: "k".to_owned(),
            },
            Keybind {
                command: BaseCommand::MoveWindowDown,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned(), "Shift".to_owned()].into()),
                key: "j".to_owned(),
            },
            Keybind {
                command: BaseCommand::MoveWindowTop,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned()].into()),
                key: "Return".to_owned(),
            },
            Keybind {
                command: BaseCommand::FocusWindowUp,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned()].into()),
                key: "k".to_owned(),
            },
            Keybind {
                command: BaseCommand::FocusWindowDown,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned()].into()),
                key: "j".to_owned(),
            },
            Keybind {
                command: BaseCommand::NextLayout,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned(), "Control".to_owned()].into()),
                key: "k".to_owned(),
            },
            Keybind {
                command: BaseCommand::PreviousLayout,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned(), "Control".to_owned()].into()),
                key: "j".to_owned(),
            },
            Keybind {
                command: BaseCommand::FocusWorkspaceNext,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned()].into()),
                key: "l".to_owned(),
            },
            Keybind {
                command: BaseCommand::FocusWorkspacePrevious,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned()].into()),
                key: "h".to_owned(),
            },
            Keybind {
                command: BaseCommand::MoveWindowUp,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned(), "Shift".to_owned()].into()),
                key: "Up".to_owned(),
            },
            Keybind {
                command: BaseCommand::MoveWindowDown,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned(), "Shift".to_owned()].into()),
                key: "Down".to_owned(),
            },
            Keybind {
                command: BaseCommand::FocusWindowUp,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned()].into()),
                key: "Up".to_owned(),
            },
            Keybind {
                command: BaseCommand::FocusWindowDown,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned()].into()),
                key: "Down".to_owned(),
            },
            Keybind {
                command: BaseCommand::NextLayout,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned(), "Control".to_owned()].into()),
                key: "Up".to_owned(),
            },
            Keybind {
                command: BaseCommand::PreviousLayout,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned(), "Control".to_owned()].into()),
                key: "Down".to_owned(),
            },
            Keybind {
                command: BaseCommand::FocusWorkspaceNext,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned()].into()),
                key: "Right".to_owned(),
            },
            Keybind {
                command: BaseCommand::FocusWorkspacePrevious,
                value: String::default(),
                modifier: Some(vec!["modkey".to_owned()].into()),
                key: "Left".to_owned(),
            },
        ];

        // add "goto workspace"
        for i in 1..WORKSPACES_NUM {
            commands.push(Keybind {
                command: BaseCommand::GotoTag,
                value: i.to_string(),
                modifier: Some(vec!["modkey".to_owned()].into()),
                key: i.to_string(),
            });
        }

        // and "move to workspace"
        for i in 1..WORKSPACES_NUM {
            commands.push(Keybind {
                command: BaseCommand::MoveToTag,
                value: i.to_string(),
                modifier: Some(vec!["modkey".to_owned(), "Shift".to_owned()].into()),
                key: i.to_string(),
            });
        }

        let tags = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();

        Self {
            workspaces: Some(vec![]),
            tags: Some(tags),
            layouts: LAYOUTS.to_vec(),
            layout_mode: LayoutMode::Workspace,
            // TODO: add sane default for scratchpad config.
            // Currently default values are set in sane_dimension fn.
            scratchpad: Some(vec![]),
            disable_current_tag_swap: false,
            disable_tile_drag: false,
            focus_behaviour: FocusBehaviour::Sloppy, // default behaviour: mouse move auto-focuses window
            focus_new_windows: true, // default behaviour: focuses windows on creation
            modkey: "Mod4".to_owned(), //win key
            mousekey: Some("Mod4".into()), //win key
            keybind: commands,
            theme_setting: ThemeSetting::default(),
            max_window_width: None,
            state: None,
            window_rules: Vec::new(),
        }
    }
}
