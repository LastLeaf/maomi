use std::path::PathBuf;

fn main() {
    if let Ok(crate_path) = std::env::var("CARGO_MANIFEST_DIR") {
        let crate_path = PathBuf::from(crate_path);

        // enable CSS output and specify the CSS output directory
        println!(
            "cargo:rustc-env=MAOMI_CSS_OUT_DIR={}",
            crate_path.join("pkg").to_str().unwrap(),
        );

        // specify the CSS output mode
        #[cfg(debug_assertions)]
        println!("cargo:rustc-env=MAOMI_CSS_OUT_MODE=debug");

        // specify the root crate CSS module (default to `src/lib.mcss`)
        println!(
            "cargo:rustc-env=MAOMI_STYLESHEET_MOD_ROOT={}",
            crate_path.join("src").join("lib.mcss").to_str().unwrap(),
        );

        // enable i18n support and specify the current locale
        println!("cargo:rerun-if-env-changed=LANG");
        if let Ok(lang) = std::env::var("LANG") {
            let locale = lang.split('.').next().unwrap();
            println!(
                "cargo:rustc-env=MAOMI_I18N_LOCALE={}",
                locale,
            );
        }

        // specify where to find the locale files
        println!(
            "cargo:rustc-env=MAOMI_I18N_DIR={}",
            crate_path.join("i18n").to_str().unwrap(),
        );
    }
}
