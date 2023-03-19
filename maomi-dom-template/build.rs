fn main() {
    // rerun if locale changes
    println!("cargo:rerun-if-env-changed=MAOMI_I18N_LOCALE");
}
