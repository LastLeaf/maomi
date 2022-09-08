use std::path::PathBuf;

fn main() {
    if let Ok(crate_path) = std::env::var("CARGO_MANIFEST_DIR") {
        #[cfg(debug_assertions)]
        println!("cargo:rustc-env=MAOMI_CSS_OUT_MODE=debug");
        let crate_path = PathBuf::from(crate_path);
        println!(
            "cargo:rustc-env=MAOMI_CSS_IMPORT_DIR={}",
            crate_path.to_str().unwrap()
        );
    }
}
