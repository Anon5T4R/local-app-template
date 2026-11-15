//! Interface languages, on `craft-i18n` (vendored in `libs/craft-libs/crates/i18n`).
//!
//! Strings in the code stay English and are the lookup keys; the catalogs under
//! `i18n/locales/<code>/` map them to display text when the UI is drawn. A string without a
//! translation shows in English, so coverage can grow one entry at a time, and a missing
//! translation never fails a build, a test or a release. The catalogs are embedded by `build.rs`
//! (`CRAFT_I18N_DIR` overrides the directory; releases set `CRAFT_I18N_REQUIRED=1`).
//!
//! Adding a language: drop `i18n/locales/<code>/localapp.tsv` in, add one row to [`LANGUAGES`],
//! and the picker, the system-language match and the fallback all pick it up.

use std::cell::Cell;
use std::sync::LazyLock;

use craft_i18n::I18n;

include!(concat!(env!("OUT_DIR"), "/craft_i18n.rs"));

/// The `language` setting that follows the operating system's language.
pub const AUTO: &str = "auto";

/// One interface language this app offers. Its display name and translations come from
/// craft-i18n (the catalog's `@id language.name` row, else its code).
pub struct LangInfo {
    /// BCP 47 code, lowercase (`pt-br`). Also the `language` setting's value and the craft-i18n
    /// locale.
    pub code: &'static str,
}

/// The languages this app offers, when their catalogs are embedded. English first: it is the
/// fallback and the source language.
pub static LANGUAGES: [LangInfo; 4] = [
    LangInfo { code: "en" },
    // Brazilian Portuguese; `pt-BR` and `pt-BR-*` resolve here.
    LangInfo { code: "pt-br" },
    // Spanish, neutral across Spain and Latin America; `es-ES`, `es-MX` and `es-419` resolve here.
    LangInfo { code: "es" },
    // Japanese: the reason a release build embeds the craft-fonts CJK faces.
    LangInfo { code: "ja" },
];

/// The embedded catalogs, offering exactly the languages in [`LANGUAGES`]. Built on first use; a
/// locale is parsed on its first lookup.
static I18N: LazyLock<I18n> = LazyLock::new(|| {
    let offered: Vec<&str> = LANGUAGES.iter().map(|l| l.code).collect();
    let (i18n, problems) = I18n::new(CRAFT_I18N, &offered);
    problems.iter().for_each(|p| log::warn!("interface catalogs: {p}"));
    i18n
});

thread_local! {
    /// The language of the current frame. The UI runs on the main thread, so a thread-local keeps
    /// every widget's lookup free of arguments.
    static CURRENT: Cell<craft_i18n::Lang> = const { Cell::new(craft_i18n::Lang::EN) };
}

/// The language `setting` asks for: `auto` (the system's language, English when there is no
/// catalog for it) or a code from [`LANGUAGES`].
pub fn resolve(setting: &str) -> craft_i18n::Lang {
    if setting == AUTO {
        return I18N.system_lang();
    }
    I18N.match_tag(setting).or_else(|| I18N.from_code(setting)).unwrap_or(craft_i18n::Lang::EN)
}

/// Sets the language of the following frames. Call it at start-up (before the first frame) and
/// whenever the language setting changes.
pub fn set_language(setting: &str) {
    let lang = resolve(setting);
    CURRENT.with(|current| current.set(lang));
}

/// Sets the language from an already resolved [`craft_i18n::Lang`] (the picker's choice).
pub fn set_lang(lang: craft_i18n::Lang) {
    CURRENT.with(|current| current.set(lang));
}

/// The language of this frame.
pub fn current() -> craft_i18n::Lang {
    CURRENT.with(Cell::get)
}

/// `source` (English) in the current language, else in English.
pub fn tr(source: &str) -> &str {
    I18N.t(current(), source)
}

/// The code of the current language (`en`, `pt-br`).
pub fn code() -> &'static str {
    I18N.code(current())
}

/// The offered languages, with the name each one shows under (in its own language).
pub fn languages() -> Vec<(&'static str, &'static str)> {
    LANGUAGES.iter().map(|l| (l.code, name_of(l.code))).collect()
}

/// A language's display name from its catalog's `@id language.name` row, else its code.
pub fn name_of(code: &str) -> &'static str {
    I18N.from_code(code).map(|lang| I18N.name(lang)).unwrap_or_else(|| LANGUAGES.iter().find(|l| l.code == code).map_or("English", |l| l.code))
}

/// `source` (English) in the current language, with `{name}` placeholders filled.
pub fn tr_fmt(source: &str, args: &[(&str, &str)]) -> String {
    craft_i18n::fmt(tr(source), args)
}

/// A string in the current language: `tl!("About")`.
#[macro_export]
macro_rules! tl {
    ($s:expr) => {
        $crate::i18n::tr($s)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catalog_parses() {
        // A malformed row or locale is a warning at run time (the English text shows); the gate
        // here catches it before it ships.
        let problems = I18N.check_all();
        assert!(problems.is_empty(), "{problems:#?}");
    }

    #[test]
    fn english_is_the_fallback_and_pt_br_falls_back_to_english() {
        set_language("en");
        assert_eq!(tr("About"), "About");
        set_language("pt-br");
        assert_eq!(code(), "pt-br");
        // Every language falls back to the English source for a string it does not translate.
        assert_eq!(tr("a string no catalog has"), "a string no catalog has");
        set_language(AUTO);
    }

    #[test]
    fn a_language_the_app_does_not_offer_resolves_to_english() {
        assert_eq!(resolve("tlh"), craft_i18n::Lang::EN);
        assert_eq!(resolve("xx-YY"), craft_i18n::Lang::EN);
    }
}
