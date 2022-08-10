use std::path::PathBuf;

fn main() {
    if let Ok(crate_path) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(crate_path);
        path.push("pkg");
        println!("cargo:rustc-env=MAOMI_CSS_OUT_DIR={}", path.to_str().unwrap());
        #[cfg(debug_assertions)]
        println!("cargo:rustc-env=MAOMI_CSS_OUT_MODE=debug");
    }
}
