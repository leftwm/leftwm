use std::env;

fn main() {
    // NOTE: this is what should be used
    let features: Vec<String> = env::vars()
        .filter_map(|(name, _)| if name.starts_with("CARGO_FEATURE_") {
            let name = name[14..].replace("_", "-");
            let name = name.to_lowercase();
            Some(name)
        } else {
            None
        })
        .collect();

    println!("cargo:warning={:?}", features);
    println!("cargo:rustc-env=LEFTWM_FEATURES={:?}", features);

    println!("cargo:warning=When first time building with `lefthk` you need to completely restart `leftwm` in order to start the hotkey daemon proprerly. A `SoftReload` or `HardReload` will leave you with a session non responsive to keybinds but otherwise running well.");
}
