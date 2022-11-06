use std::env;

fn main() {
    let mut features_string = String::new();
    env::vars()
        .for_each(|(name, _)| if name.starts_with("CARGO_FEATURE_") {
            let name = name[14..].replace("_", "-");
            let name = name.to_lowercase();
            features_string.push(' ');
            features_string.push_str(&name);
        });

    println!("cargo:rustc-env=LEFTWM_FEATURES={}", features_string);

    println!("cargo:warning=When first time building with `lefthk` you need to completely restart `leftwm` in order to start the hotkey daemon proprerly. A `SoftReload` or `HardReload` will leave you with a session non responsive to keybinds but otherwise running well.");
}
