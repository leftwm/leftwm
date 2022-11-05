use std::env;

fn main() {
    // NOTE: this is what should be used
    let features: Vec<String> = env::args()
        .filter(|arg| arg.starts_with("CARGO_FEATURE"))
        .collect();

    println!("cargo:warning=When first time building with `lefthk` you need to completely restart `leftwm` in order to start the hotkey daemon proprerly. A `SoftReload` or `HardReload` will leave you with a session non responsive to keybinds but otherwise running well.");
}
