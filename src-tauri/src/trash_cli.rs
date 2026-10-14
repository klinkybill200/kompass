//! Kommandozeilen-Modus der App (ohne Fenster):
//!
//!   kompass --kompass-trash -- datei1 datei2 …   → in den Papierkorb (Ersatz für rm)
//!   kompass --kompass-undo                        → letzten Papierkorb-Vorgang rückgängig machen
//!
//! Aufgerufen von den zsh-Funktionen `kompass-trash` und `kompass-undo`.
//! Die eigentliche Arbeit macht trashlog.rs – hier geht es nur um die Ausgabe.

use crate::i18n::tr;
use crate::trashlog::{self, Entry};
use std::path::PathBuf;

// ANSI-Farbcodes für die Terminal-Ausgabe.
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

/// `kompass --kompass-trash -- …`. Gibt den Exit-Code zurück (0 = alles gut).
pub fn trash(args: &[String]) -> i32 {
    // Alles nach `--` sind Dateinamen. `position` sucht den Index von "--".
    let names: Vec<&String> = match args.iter().position(|a| a == "--") {
        Some(i) => args[i + 1..].iter().collect(),
        None => args.iter().collect(),
    };
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let batch = trashlog::now_ms();
    let mut entries = Vec::new();
    let mut problems = 0;

    for name in names {
        // Pfad relativ zum aktuellen Ordner – Verknüpfungen werden NICHT aufgelöst,
        // damit nur die Verknüpfung selbst im Papierkorb landet, nicht ihr Ziel.
        let path = cwd.join(name);
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            println!("{YELLOW}{}{RESET}", t!("∅  „{name}“ gibt es hier nicht – übersprungen.", "∅  “{name}” does not exist here – skipped."));
            problems += 1;
            continue;
        };
        let what = if meta.is_dir() { tr("Ordner", "Folder") } else { tr("Datei", "File") };
        match trashlog::move_to_trash(&path) {
            Ok(trashed) => {
                println!("{GREEN}{}{RESET}", t!("🗑  {what} „{name}“ liegt jetzt im Papierkorb.", "🗑  {what} “{name}” is now in the Trash."));
                entries.push(Entry { batch, original: path, trashed });
            }
            Err(e) => {
                println!("{YELLOW}{}{RESET}", t!("✗  „{name}“ konnte nicht in den Papierkorb: {e}", "✗  “{name}” could not be moved to the Trash: {e}"));
                problems += 1;
            }
        }
    }

    if !entries.is_empty() {
        if let Err(e) = trashlog::append_log(&entries) {
            println!("{YELLOW}{}{RESET}", t!("   (Hinweis: Rückgängig-Protokoll nicht gespeichert: {e})", "   (Note: undo log not saved: {e})"));
        }
        println!("{DIM}{}{RESET}kompass-undo{DIM}{}{RESET}", tr("   Rückgängig: tippe ", "   Undo: type "), tr(" – oder hol sie dir aus dem Papierkorb im Dock.", " – or get them back from the Trash in the Dock."));
        // Läuft das in Kompass, dem Fenster unsichtbar Bescheid geben (→ „Rückgängig“-Knopf).
        if std::env::var("TERM_PROGRAM").as_deref() == Ok("Kompass") {
            print!("\x1b]7773;{batch}\x07");
        }
    }
    if problems > 0 { 1 } else { 0 }
}

/// `kompass --kompass-undo`: legt die Dateien des letzten Vorgangs zurück.
pub fn undo() -> i32 {
    let Some(batch) = trashlog::last_batch() else {
        println!("{YELLOW}{}{RESET}", tr("Es gibt nichts rückgängig zu machen.", "There is nothing to undo."));
        return 1;
    };
    let mut problems = 0;
    for r in trashlog::restore_batch(batch) {
        if r.ok {
            println!("{GREEN}↩  {} {}{RESET}", crate::explain::quote(&r.name), r.message);
        } else {
            println!("{YELLOW}✗  {} {}{RESET}", crate::explain::quote(&r.name), r.message);
            problems += 1;
        }
    }
    if problems > 0 { 1 } else { 0 }
}
