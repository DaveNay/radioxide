use fluent_bundle::{FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

const EN_US_FTL: &str = include_str!("../resources/locales/en-US/radio.ftl");

pub struct I18n {
    bundle: FluentBundle<FluentResource>,
}

impl I18n {
    pub fn new() -> Self {
        let locale = detect_locale();
        let ftl_source = select_locale(&locale);

        let langid: LanguageIdentifier = locale.parse().unwrap_or_else(|_| "en-US".parse().unwrap());
        let resource =
            FluentResource::try_new(ftl_source.to_string()).expect("failed to parse FTL resource");

        let mut bundle = FluentBundle::new(vec![langid]);
        bundle
            .add_resource(resource)
            .expect("failed to add FTL resource to bundle");

        Self { bundle }
    }

    /// Look up a localized string by key.
    pub fn t(&self, key: &str) -> String {
        let msg = match self.bundle.get_message(key) {
            Some(m) => m,
            None => return key.to_string(),
        };
        let pattern = match msg.value() {
            Some(p) => p,
            None => return key.to_string(),
        };
        let mut errors = vec![];
        self.bundle
            .format_pattern(pattern, None, &mut errors)
            .to_string()
    }
}

fn detect_locale() -> String {
    sys_locale::get_locale().unwrap_or_else(|| "en-US".to_string())
}

fn select_locale(_locale: &str) -> &'static str {
    // For now, only en-US is bundled. Future locales can be added here.
    EN_US_FTL
}
