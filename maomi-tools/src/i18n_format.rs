use std::path::PathBuf;
use rustc_hash::{FxHashMap, FxHashSet};
use clap::Parser;

use maomi_tools::i18n::{Locale, METADATA_VERSION};

#[derive(serde::Deserialize)]
struct FormatMetadataOwned {
    version: u32,
    item: Vec<FormatMetadataItemOwned>,
}

#[derive(serde::Deserialize)]
struct FormatMetadataItemOwned {
    namespace: String,
    src: String,
    translated: Option<String>,
}

fn toml_str_escape(s: &str) -> String {
    s.chars().map(|x| {
        match x {
            '\u{0008}' => "\\b".to_string(),
            '\t' => "\\t".to_string(),
            '\n' => "\\n".to_string(),
            '\u{000C}' => "\\f".to_string(),
            '\r' => "\\r".to_string(),
            '\"' => "\\\"".to_string(),
            '\\' => "\\\\".to_string(),
            x => x.to_string(),
        }
    }).collect()
}

fn do_format(
    w: &mut impl std::fmt::Write,
    format_metadata: FormatMetadataOwned,
    mut src: Locale,
    missing_sign: Option<&str>,
) -> Result<(), std::fmt::Error> {
    struct TransItem<'a> {
        src: &'a str,
        translated: &'a str,
        missing: bool,
        unused: bool,
    }

    // group by namespaces
    let mut namespaces = vec!["translation"];
    let mut map: FxHashMap<&str, (FxHashSet<&str>, Vec<TransItem>)> = FxHashMap::default();
    map.insert("translation", (FxHashSet::default(), vec![]));
    for item in &format_metadata.item {
        let (set, arr) = map.entry(&item.namespace).or_insert_with(|| {
            namespaces.push(&item.namespace);
            (FxHashSet::default(), vec![])
        });
        if !set.insert(&item.src) { continue };
        arr.push(TransItem {
            src: &item.src,
            translated: item.translated.as_ref().map(|x| x.as_str()).unwrap_or_default(),
            missing: item.translated.is_none(),
            unused: false,
        });
        if let Some(x) = src.get_mut(&item.namespace) {
            x.remove(&item.src);
        }
    }

    // add unused translations
    for (ns, trans) in &src {
        if trans.is_empty() { continue };
        for (src, translated) in trans {
            let (_, arr) = map.entry(&ns).or_insert_with(|| {
                namespaces.push(&ns);
                (FxHashSet::default(), vec![])
            });
            arr.push(TransItem {
                src,
                translated,
                missing: false,
                unused: true,
            });
        }
    }

    // write translation
    writeln!(w, "# formatted by maomi-i18n-format")?;
    for ns in namespaces {
        let (_, trans_items) = map.get(ns).unwrap();
        writeln!(w, "\n[{}]", ns)?;
        for item in trans_items {
            if item.unused {
                writeln!(w, "# (unused)")?;
            } else if item.missing {
                if let Some(sign) = missing_sign.clone() {
                    write!(w, "# {} # ", sign)?;
                } else {
                    write!(w, "# ")?;
                }
            }
            writeln!(w, r#""{}" = "{}""#, toml_str_escape(item.src), toml_str_escape(item.translated))?;
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Format a translation file for maomi")]
struct CmdArgs {
    /// The locale to format
    #[arg(short, long)]
    locale: Option<String>,
    /// Add a comment for missing translation
    #[arg(short, long)]
    missing: Option<String>,
    /// Output to stdout instead of the translation file
    #[arg(long)]
    print: bool,
    /// The crate path (default to working directory)
    dir: Option<PathBuf>,
}

fn main() {
    let cmd_args = CmdArgs::parse();

    // locate the crate by Cargo.toml
    let mut cur_dir = std::env::current_dir().unwrap_or_default();
    if let Some(p) = cmd_args.dir.as_ref() {
        cur_dir.push(p);
    }
    if !cur_dir.join("Cargo.toml").exists() {
        panic!("Cargo.toml not found at {:?}", cur_dir);
    }
    std::env::set_var("CARGO_MANIFEST_DIR", cur_dir);

    maomi_tools::config::crate_config(|crate_config| {
        // read config and do format
        let locale = {
            cmd_args.locale.as_ref().unwrap_or_else(|| {
                crate_config.i18n_locale.as_ref().expect("locale not specified (try specify `MAOMI_I18N_LOCALE` environment variable)")
            })
        };
        let i18n_dir = crate_config.i18n_dir.as_ref().expect("no proper i18n directory found");

        // read metadata and original translation file
        let src_path = i18n_dir.join(format!("{}.toml", locale));
        let format_metadata_path = i18n_dir.join("format-metadata").join(format!("{}.toml", locale));
        let format_metadata = std::fs::read_to_string(&format_metadata_path).expect("no format metadata found (try build this crate with environment variable `MAOMI_I18N_FORMAT_METADATA=on`)");
        let format_metadata: FormatMetadataOwned = toml::from_str(&format_metadata).expect("illegal format metadata");
        if format_metadata.version != METADATA_VERSION {
            panic!("the format metadata is generated by a different version of maomi");
        }
        let src = std::fs::read_to_string(&src_path).unwrap_or_default();
        let src: Locale = toml::from_str(&src).unwrap_or_default();

        // do the formatting
        let mut r = String::new();
        do_format(&mut r, format_metadata, src, cmd_args.missing.as_ref().map(|x| x.as_str())).unwrap();
        let formatted = r;

        // output
        if cmd_args.print {
            println!("{}", formatted);
        } else {
            std::fs::write(&src_path, &formatted).expect("Failed to write formatted content");
        }
    });
}
