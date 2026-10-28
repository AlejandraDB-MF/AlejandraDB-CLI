use crate::config::Lang;

pub fn init(lang: Lang) {
    set_language(lang);
}

pub fn set_language(lang: Lang) {
    rust_i18n::set_locale(lang.code());
}

pub fn tr(key: &str) -> String {
    rust_i18n::t!(key).into_owned()
}
