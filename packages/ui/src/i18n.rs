use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Language {
    En,
    Zh,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Zh => "zh",
        }
    }
}

pub fn use_i18n_provider() {
    use_context_provider(|| Signal::new(Language::Zh));
}

pub fn use_language() -> Signal<Language> {
    use_context::<Signal<Language>>()
}
