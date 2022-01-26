use super::Config;
use leftwm_core::utils;
use std::collections::HashSet;

impl Config {
    pub fn check_mousekey(&self, verbose: bool) {
        if verbose {
            println!("Checking if mousekey is set.");
        }
        if let Some(mousekey) = &self.mousekey {
            if verbose {
                println!("Mousekey is set.");
            }
            if *mousekey == "".into()
                || *mousekey == vec!["".to_owned()].into()
                || *mousekey == vec![].into()
            {
                println!("Your mousekey is set to nothing, this will cause windows to move/resize with just a mouse press.");
                return;
            }
            if verbose {
                println!("Mousekey is okay.");
            }
        }
    }

    /// Checks defined workspaces to ensure no ID collisions occur.
    pub fn check_workspace_ids(&self, verbose: bool) {
        if let Some(wss) = self.workspaces.as_ref() {
            if verbose {
                println!("Checking config for valid workspace definitions.");
            }
            let ids = crate::get_workspace_ids(wss);
            if ids.iter().any(std::option::Option::is_some) {
                if crate::all_ids_some(&ids) && !crate::all_ids_unique(&ids) {
                    println!("Your config.toml contains duplicate workspace IDs. Please assign unique IDs to workspaces. The default config will be used instead.");
                } else {
                    println!("Your config.toml specifies an ID for some but not all workspaces. This can lead to ID collisions and is not allowed. The default config will be used instead.");
                }
            }
        }
    }

    /// Check all keybinds to ensure that required values are provided
    /// Checks to see if value is provided (if required)
    /// Checks to see if keys are valid against Xkeysym
    /// Ideally, we will pass this to the command handler with a dummy config
    pub fn check_keybinds(&self, verbose: bool) {
        let mut returns = Vec::new();
        println!("\x1b[0;94m::\x1b[0m Checking keybinds . . .");
        let mut bindings = HashSet::new();
        for keybind in &self.keybind {
            if verbose {
                println!("Keybind: {:?} {}", keybind, keybind.value.is_empty());
            }
            if let Err(err) = keybind.try_convert_to_core_keybind(self) {
                returns.push((Some(keybind.clone()), err.to_string()));
            }
            if utils::xkeysym_lookup::into_keysym(&keybind.key).is_none() {
                returns.push((
                    Some(keybind.clone()),
                    format!("Key `{}` is not valid", keybind.key),
                ));
            }

            let mut modkey = keybind.modifier.as_ref().unwrap_or(&"None".into()).clone();
            for m in &modkey.clone() {
                if m != "modkey" && m != "mousekey" && utils::xkeysym_lookup::into_mod(&m) == 0 {
                    returns.push((
                        Some(keybind.clone()),
                        format!("Modifier `{}` is not valid", m),
                    ));
                }
            }

            modkey.sort_unstable();
            if let Some(conflict_key) = bindings.replace((modkey.clone(), &keybind.key)) {
                returns.push((
                    None,
                    format!(
                        "\x1b[0m\x1b[1mMultiple commands bound to key combination {} + {}:\
                    \n\x1b[1;91m    -> {:?}\
                    \n    -> {:?}\
                    \n\x1b[0mHelp: change one of the keybindings to something else.\n",
                        modkey, keybind.key, conflict_key, keybind.command,
                    ),
                ));
            }
        }
        if returns.is_empty() {
            println!("\x1b[0;92m    -> All keybinds OK\x1b[0m");
        } else {
            for error in returns {
                match error.0 {
                    Some(binding) => {
                        println!(
                            "\x1b[1;91mERROR: {} for keybind {:?}\x1b[0m",
                            error.1, binding
                        );
                    }
                    None => {
                        println!("\x1b[1;91mERROR: {} \x1b[0m", error.1);
                    }
                }
            }
        }
    }
}
