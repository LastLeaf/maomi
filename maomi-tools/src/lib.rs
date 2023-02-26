pub mod config;

pub mod i18n {
    use rustc_hash::FxHashMap;

    pub type Locale = FxHashMap<String, FxHashMap<String, String>>;

    #[derive(serde::Serialize)]
    pub struct FormatMetadata<'a> {
        pub item: Vec<FormatMetadataItem<'a>>,
    }

    #[derive(serde::Serialize)]
    pub struct FormatMetadataItem<'a> {
        pub namespace: &'a str,
        pub src: &'a str,
        pub translated: Option<&'a str>,
    }
}
