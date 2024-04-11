use std::env;

fn main() {
    #[cfg(all(not(feature = "x11rb"), not(feature = "xlib")))]
    compile_error!("You need to build with at least one backend feature.");

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
        Err(_) => println!("cargo:warning=When building with `lefthk` for the first time, you will need to completely restart `leftwm` in order to start the hotkey daemon properly. A `SoftReload` or `HardReload` will leave you with a session that is not responsive to keybinds but otherwise running well."),
    }
}
