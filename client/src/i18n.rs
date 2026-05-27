use std::collections::HashMap;
use std::cell::RefCell;

thread_local! {
    static TRANSLATIONS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    static CURRENT_LANG: RefCell<String> = RefCell::new("es".to_string());
}

const SUPPORTED_LANGS: &[&str] = &["es", "en", "fr", "de", "ja"];
const LANG_NAMES: &[(&str, &str)] = &[
    ("es", "Español"),
    ("en", "English"),
    ("fr", "Français"),
    ("de", "Deutsch"),
    ("ja", "日本語"),
];

pub fn init() {
    let lang = detect_lang();
    set_lang(&lang);
}

pub fn supported_langs() -> &'static [&'static str] {
    SUPPORTED_LANGS
}

pub fn lang_name(code: &str) -> &'static str {
    for &(c, n) in LANG_NAMES {
        if c == code { return n; }
    }
    "Unknown"
}

pub fn current_lang() -> String {
    CURRENT_LANG.with(|l| l.borrow().clone())
}

pub fn set_lang(code: &str) {
    let code = if SUPPORTED_LANGS.contains(&code) { code.to_string() } else { "en".to_string() };
    let data = match code.as_str() {
        "es" => include_str!("../i18n/es.json"),
        "en" => include_str!("../i18n/en.json"),
        "fr" => include_str!("../i18n/fr.json"),
        "de" => include_str!("../i18n/de.json"),
        "ja" => include_str!("../i18n/ja.json"),
        _ => include_str!("../i18n/en.json"),
    };
    let parsed: HashMap<String, String> = serde_json::from_str(data).unwrap_or_default();
    TRANSLATIONS.with(|t| *t.borrow_mut() = parsed);
    CURRENT_LANG.with(|l| *l.borrow_mut() = code);
}

pub fn t(key: &str) -> String {
    TRANSLATIONS.with(|t| {
        t.borrow().get(key).cloned().unwrap_or_else(|| {
            // Fallback: try English
            let en: HashMap<String, String> = serde_json::from_str(include_str!("../i18n/en.json")).unwrap_or_default();
            en.get(key).cloned().unwrap_or_else(|| {
                // Last fallback: return key with brackets
                format!("[{}]", key)
            })
        })
    })
}

fn detect_lang() -> String {
    if let Some(win) = web_sys::window() {
        let lang = win.navigator().language().unwrap_or_else(|| "en".to_string());
        let code = lang.split('-').next().unwrap_or("en").to_lowercase();
        if SUPPORTED_LANGS.contains(&code.as_str()) {
            return code;
        }
    }
    "en".to_string()
}
