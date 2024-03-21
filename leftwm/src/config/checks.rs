use super::Config;
#[cfg(feature = "lefthk")]
use lefthk_core::xkeysym_lookup;
#[cfg(feature = "lefthk")]
use std::collections::HashSet;
use tracing_subscriber::EnvFilter;

impl Config {
    pub fn check_mousekey(&self, verbose: bool) {
        if verbose {
            println!("Checking if mousekey is set.");
        }
        if let Some(mousekey) = &self.mousekey {
            if verbose {
                println!("Mousekey is set.");
            }
            if mousekey.is_empty() {
                println!("Your mousekey is set to nothing, this will cause windows to move/resize with just a mouse press.");
                return;
            }
            if verbose {
                println!("Mousekey is okay.");
            }
        }
    }

    pub fn check_log_level(&self, verbose: bool) {
        if verbose {
            println!("Trying to parse log_level.");
        }
        match EnvFilter::builder().parse(&self.log_level) {
            Ok(_) if verbose => println!("Log level is ok."),
            Ok(_) => {}
            Err(err) => println!("Log level is invalid: {err}"),
        }
    }

    /// Check all keybinds to ensure that required values are provided
    /// Checks to see if value is provided (if required)
    /// Checks to see if keys are valid against Xkeysym
    /// Ideally, we will pass this to the command handler with a dummy config
    #[cfg(feature = "lefthk")]
    pub fn check_keybinds(&self, verbose: bool) {
        let mut returns = Vec::new();
        println!("\x1b[0;94m::\x1b[0m Checking keybinds . . .");
        let mut bindings = HashSet::new();
        for keybind in &self.keybind {
            if verbose {
                println!(
                    "Keybind: {:?} value field is empty: {}",
                    keybind,
                    keybind.value.is_empty()
                );
            }
            if let Err(err) = keybind.try_convert_to_lefthk_keybind(self) {
                returns.push((Some(keybind.clone()), err.to_string()));
            }
            if xkeysym_lookup::into_keysym(&keybind.key).is_none() {
                returns.push((
                    Some(keybind.clone()),
                    format!("Key `{}` is not valid", keybind.key),
                ));
            }

            let mut modkey = keybind.modifier.as_ref().unwrap_or(&"None".into()).clone();
            for m in &modkey.clone() {
                if m != "modkey" && m != "mousekey" && xkeysym_lookup::into_mod(&m) == 0 {
                    returns.push((
                        Some(keybind.clone()),
                        format!("Modifier `{m}` is not valid"),
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
                            "\x1b[1;91mERROR: {} for keybind {binding:?}\x1b[0m",
                            error.1
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
