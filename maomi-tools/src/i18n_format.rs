#[derive(serde::Serialize, serde::Deserialize)]
struct FormatMetadataOwned {
    item: Vec<FormatMetadataItemOwned>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct FormatMetadataItemOwned {
    namespace: String,
    src: String,
    translated: Option<String>,
}

fn main() {
    let cur_dir = std::env::current_dir().unwrap_or_default();
    std::env::set_var("CARGO_MANIFEST_DIR", cur_dir);
    maomi_tools::config::crate_config(|crate_config| {
        let locale = crate_config.i18n_locale.as_ref().expect("MAOMI_I18N_LOCALE environment variable not set");
        let i18n_dir = crate_config.i18n_dir.as_ref().expect("no proper i18n directory found");
        let src_path = i18n_dir.join("format-metadata").join(format!("{}.toml", locale));
        let format_metadata_path = i18n_dir.join("format-metadata").join(format!("{}.toml", locale));
        let format_metadata = std::fs::read_to_string(&format_metadata_path).expect("no format metadata found (try build this crate with environment variable `MAOMI_I18N_FORMAT_METADATA=on`)");
        let format_metadata: FormatMetadataOwned = toml::from_str(&format_metadata).expect("illegal format metadata");
        let src = std::fs::read_to_string(&src_path).unwrap_or_default();
        let src: Vec<(String, Vec<(String, String)>)> = toml::from_str(&src).unwrap_or_default();
        
    });
}
