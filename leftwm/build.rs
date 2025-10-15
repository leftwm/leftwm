use std::env;

fn main() {
    #[cfg(all(
        not(feature = "x11rb"),
        not(feature = "xlib"),
        not(feature = "smithay")
    ))]
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
}
