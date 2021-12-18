use super::*;
use leftwm_core::utils;
use std::collections::HashSet;

impl Config {
    /// Checks defined workspaces to ensure no ID collisions occur.
    pub fn check_workspace_ids(&self, verbose: bool) -> bool {
        self.workspaces.as_ref().map_or(true, |wss|
    {
        if verbose {
            println!("Checking config for valid workspace definitions.");
        }
        let ids = crate::get_workspace_ids(wss);
        if ids.iter().any(std::option::Option::is_some) {
            if !crate::all_ids_some(&ids)
            {
                println!("Your config.toml specifies an ID for some but not all workspaces. This can lead to ID collisions and is not allowed. The default config will be used instead.");
                false
            } else if crate::all_ids_unique(&ids) {
                true
            } else {
                println!("Your config.toml contains duplicate workspace IDs. Please assign unique IDs to workspaces. The default config will be used instead.");
                false
            }
        } else {
            true
        }
    }
    )
    }

    /// Check all keybinds to ensure that required values are provided
    /// Checks to see if value is provided (if required)
    /// Checks to see if keys are valid against Xkeysym
    /// Ideally, we will pass this to the command handler with a dummy config
    pub fn check_keybinds(&self, verbose: bool) -> bool {
        let mut returns = Vec::new();
        println!("\x1b[0;94m::\x1b[0m Checking keybinds . . .");
        let mut bindings = HashSet::new();
        for keybind in self.keybind.iter() {
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

            for m in &keybind.modifier {
                if m != "modkey" && m != "mousekey" && utils::xkeysym_lookup::into_mod(m) == 0 {
                    returns.push((
                        Some(keybind.clone()),
                        format!("Modifier `{}` is not valid", m),
                    ));
                }
            }
            let mut modkey = keybind.modifier.clone();
            modkey.sort_unstable();
            if let Some(conflict_key) = bindings.replace((modkey, &keybind.key)) {
                returns.push((
                    None,
                    format!(
                        "\x1b[0m\x1b[1mMultiple commands bound to key combination {} + {}:\
                    \n\x1b[1;91m    -> {:?}\
                    \n    -> {:?}\
                    \n\x1b[0mHelp: change one of the keybindings to something else.\n",
                        keybind.modifier.join(" + "),
                        keybind.key,
                        conflict_key,
                        keybind.command,
                    ),
                ));
            }
        }
        if returns.is_empty() {
            println!("\x1b[0;92m    -> All keybinds OK\x1b[0m");
            true
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
            false
        }
    }
}
