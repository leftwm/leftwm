use std::env;

fn main() {
    let mut features_string = String::new();
    env::vars().for_each(|(name, _)| {
        if let Some(name) = name.strip_prefix("CARGO_FEATURE_") {
            let name = name.replace('_', "-");
            let name = name.to_lowercase();
            let name = name.replace("default", "");
            features_string.push(' ');
            features_string.push_str(&name);
        }
    });

    println!("cargo:rustc-env=LEFTWM_FEATURES={features_string}");

    #[cfg(all(feature = "lefthk", not(target_os = "netbsd")))]
    match std::process::Command::new("lefthk-worker").spawn() {
        Ok(mut p) => p.kill().unwrap(),
        Err(_) => println!("cargo:warning=When first time building with `lefthk` you need to completely restart `leftwm` in order to start the hotkey daemon proprerly. A `SoftReload` or `HardReload` will leave you with a session non responsive to keybinds but otherwise running well."),
    }
}
