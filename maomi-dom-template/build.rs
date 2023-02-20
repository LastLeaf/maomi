fn main() {
    // enable i18n support and get the current locale from `LANG` environment variable
    println!("cargo:rerun-if-env-changed=LANG");
    if let Ok(lang) = std::env::var("LANG") {
        let locale = lang.split('.').next().unwrap();
        println!(
            "cargo:rustc-env=MAOMI_I18N_LOCALE={}",
            locale,
        );
    }
}
