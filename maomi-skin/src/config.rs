use std::{env, path::PathBuf};

#[derive(Debug, Clone)]
pub struct CrateConfig {
    pub crate_name: Option<String>,
    pub css_out_dir: Option<PathBuf>,
    pub css_out_mode: CssOutMode,
    pub stylesheet_mod_root: Option<PathBuf>,
    pub i18n_locale: Option<String>,
    pub i18n_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CssOutMode {
    Release,
    Debug,
}

#[derive(serde::Deserialize)]
struct MaomiManifestCargo {
    package: MaomiManifestPackage,
}

#[derive(serde::Deserialize)]
struct MaomiManifestPackage {
    metadata: MaomiManifestMetadata,
}

#[derive(serde::Deserialize)]
struct MaomiManifestMetadata {
    maomi: MaomiManifest,
}

#[derive(serde::Deserialize, Debug, Default)]
struct MaomiManifest {
    #[serde(default)]
    css_out_dir: Option<String>,
    #[serde(default)]
    css_out_mode: Option<String>,
    #[serde(default)]
    stylesheet_mod_root: Option<String>,
    #[serde(default)]
    i18n_dir: Option<String>,
}

thread_local! {
    static CRATE_CONFIG: CrateConfig = {
        let crate_name = env::var("CARGO_PKG_NAME").ok();
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").ok();

        // read manifest
        let manifest = manifest_dir.as_ref().and_then(|x| {
            let mut p = PathBuf::from(x);
            p.push("Cargo.toml");
            let content = std::fs::read_to_string(&x).ok()?;
            let config: MaomiManifestCargo = toml::from_str(&content).ok()?;
            Some(config.package.metadata.maomi)
        }).unwrap_or_default();
        let MaomiManifest {
            css_out_dir,
            css_out_mode,
            stylesheet_mod_root,
            i18n_dir,
        } = manifest;

        // check env vars
        let css_out_dir = env::var("MAOMI_CSS_OUT_DIR")
            .ok()
            .or(css_out_dir)
            .map(|x| {
                let p = PathBuf::from(x);
                std::fs::create_dir_all(&p).unwrap();
                p
            });
        let css_out_mode = env::var("MAOMI_CSS_OUT_MODE")
            .ok()
            .or(css_out_mode)
            .map(|x| match x.as_str() {
                "debug" => CssOutMode::Debug,
                _ => CssOutMode::Release,
            })
            .unwrap_or(CssOutMode::Release);
        let stylesheet_mod_root = std::env::var("MAOMI_STYLESHEET_MOD_ROOT")
            .ok()
            .or(stylesheet_mod_root)
            .map(|s| PathBuf::from(&s))
            .or_else(|| {
                manifest_dir.as_ref().map(|s| PathBuf::from(&s).join("src").join("styles.mcss"))
            });
        let i18n_locale = std::env::var("MAOMI_I18N_LOCALE").ok();
        let i18n_dir = std::env::var("MAOMI_I18N_DIR")
            .ok()
            .or(i18n_dir)
            .map(|s| PathBuf::from(&s))
            .or_else(|| {
                manifest_dir.as_ref().map(|s| PathBuf::from(&s).join("i18n"))
            });

        CrateConfig {
            crate_name,
            css_out_dir,
            css_out_mode,
            stylesheet_mod_root,
            i18n_locale,
            i18n_dir,
        }
    };
}

pub fn crate_config<R>(f: impl FnOnce(&CrateConfig) -> R) -> R {
    CRATE_CONFIG.with(|c| f(c))
}
