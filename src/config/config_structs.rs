use std::default::Default;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    modkey: String,
    keybind: Vec<Keybind>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Keybind {
    command: String,
    modifier: Vec<String>,
    key: String,
}

//fn concat(s: &str, s2: &str) -> String {
//    let mut ss = s.to_owned();
//    ss.push_str(&s2.to_string());
//    ss
//}

fn concat_int(s: &str, s2: i32) -> String {
    let mut ss = s.to_owned();
    ss.push_str(&s2.to_string());
    ss
}

impl Default for Config {
    fn default() -> Self {
        let mut commands: Vec<Keybind> = vec![];

        //add goto workspace
        for i in 1..10 {
            let cmd: String = concat_int("goto_workspace_", i);
            commands.push(Keybind {
                command: cmd,
                modifier: vec!["modkey".to_owned()],
                key: i.to_string(),
            });
        }

        //add move to workspace
        for i in 1..10 {
            let cmd: String = concat_int("goto_workspace_", i);
            commands.push(Keybind {
                command: cmd,
                modifier: vec!["modkey".to_owned(), "Shift".to_owned()],
                key: i.to_string(),
            });
        }

        Config {
            modkey: "Mod4".to_owned(),
            keybind: commands,
        }
    }
}
