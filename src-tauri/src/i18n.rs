//! Sprache der Meldungen: Deutsch (Standard) oder Englisch.
//!
//! Das Frontend meldet die gewählte Sprache per `set_language`. Damit auch
//! `kompass-trash` (eigener Prozess) sie kennt, landet sie zusätzlich in einer Datei.

use std::cell::Cell;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

/// `true` = Englisch. `static` = gibt es genau einmal für die ganze App.
static ENGLISH: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// Nur für Tests: Sprache für diesen einen Thread festlegen (Tests laufen parallel).
    static OVERRIDE: Cell<Option<bool>> = const { Cell::new(None) };
}

pub fn is_en() -> bool {
    OVERRIDE.with(|o| o.get()).unwrap_or_else(|| ENGLISH.load(Ordering::Relaxed))
}

#[cfg(test)]
pub fn set_thread_en(en: bool) {
    OVERRIDE.with(|o| o.set(Some(en)));
}

/// Kurzform für Text in beiden Sprachen: `tr("Hallo", "Hello")`.
pub fn tr(de: &str, en: &str) -> String {
    if is_en() { en.to_string() } else { de.to_string() }
}

/// Wie `format!`, aber mit einer deutschen und einer englischen Vorlage:
/// `t!("Löscht {n} Dateien", "Deletes {n} files")`.
#[macro_export]
macro_rules! t {
    ($de:literal, $en:literal $(,)?) => {
        if $crate::i18n::is_en() { format!($en) } else { format!($de) }
    };
    ($de:literal, $en:literal, $($arg:tt)*) => {
        if $crate::i18n::is_en() { format!($en, $($arg)*) } else { format!($de, $($arg)*) }
    };
}

fn lang_file() -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    home.join("Library/Application Support/Kompass/sprache")
}

/// Beim Start des Kommandozeilen-Modus: gespeicherte Sprache übernehmen.
pub fn load_saved() {
    if let Ok(s) = std::fs::read_to_string(lang_file()) {
        ENGLISH.store(s.trim() == "en", Ordering::Relaxed);
    }
}

/// Vom Frontend aufgerufen, wenn die Sprache wechselt (und beim Start).
#[tauri::command]
pub fn set_language(lang: String) {
    let en = lang == "en";
    ENGLISH.store(en, Ordering::Relaxed);
    let path = lang_file();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, if en { "en" } else { "de" });
}

/// Bevorzugte Sprache des Macs („de“ oder „en“).
/// Die Webansicht meldet hier oft Englisch, egal was eingestellt ist – also fragen wir macOS selbst.
#[tauri::command]
pub fn system_language() -> String {
    let out = std::process::Command::new("/usr/bin/defaults").args(["read", "-g", "AppleLanguages"]).output();
    let text = out.map(|o| String::from_utf8_lossy(&o.stdout).to_lowercase()).unwrap_or_default();
    // Erster Eintrag der Liste, z. B. ("de-DE", "en-DE")
    let first = text.split('"').nth(1).unwrap_or("");
    if first.starts_with("de") { "de".into() } else { "en".into() }
}
