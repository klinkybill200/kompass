//! Papierkorb mit Gedächtnis.
//!
//! Kompass legt Dateien über macOS in den Papierkorb und merkt sich in
//! `~/Library/Application Support/Kompass/papierkorb.jsonl`, woher sie kamen.
//! So kann man sie per `kompass-undo` oder per Knopf im Fenster zurückholen.
//! Genutzt von der Kommandozeile (trash_cli.rs) UND vom Fenster (Tauri-Befehle unten).

use crate::i18n::tr;
use objc2_foundation::{NSFileManager, NSString, NSURL};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Eine Zeile im Protokoll. `Serialize`/`Deserialize` = in/aus JSON umwandelbar.
#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    /// Zeitstempel in Millisekunden: alle Dateien eines rm-Befehls haben denselben.
    pub batch: u64,
    pub original: PathBuf,
    pub trashed: PathBuf,
}

/// Für die Anzeige im Fenster: ein rm-Vorgang mit seinen Dateien.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Batch {
    pub batch: u64,
    pub items: Vec<Item>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub name: String,
    pub folder: String,  // Ordner, in den es zurückkommt
    pub is_dir: bool,
    pub available: bool, // liegt noch im Papierkorb?
}

/// Ergebnis beim Zurücklegen einer Datei.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Restored {
    pub name: String,
    pub ok: bool,
    pub message: String,
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64) // `as u64` wandelt die Zahl in einen anderen Typ um
        .unwrap_or(0)
}

/// Verschiebt eine Datei per macOS-Funktion in den Papierkorb und gibt
/// zurück, wo sie dort gelandet ist (bei Namensgleichheit benennt macOS sie um).
pub fn move_to_trash(path: &Path) -> Result<PathBuf, String> {
    let manager = NSFileManager::defaultManager();
    let url = NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
    // `Option<…>` als „Ausgabe-Parameter“: macOS trägt hier den neuen Ort ein.
    let mut resulting: Option<objc2::rc::Retained<NSURL>> = None;
    manager
        .trashItemAtURL_resultingItemURL_error(&url, Some(&mut resulting))
        .map_err(|e| e.localizedDescription().to_string())?;
    resulting
        .and_then(|u| u.path())
        .map(|p| PathBuf::from(p.to_string()))
        .ok_or_else(|| "Ort im Papierkorb unbekannt".to_string())
}

/// Datei verschieben – erst normal, sonst über macOS (Papierkorb ist geschützt).
fn move_item(from: &Path, to: &Path) -> Result<(), String> {
    if std::fs::rename(from, to).is_ok() {
        return Ok(());
    }
    let manager = NSFileManager::defaultManager();
    let src = NSURL::fileURLWithPath(&NSString::from_str(&from.to_string_lossy()));
    let dst = NSURL::fileURLWithPath(&NSString::from_str(&to.to_string_lossy()));
    manager
        .moveItemAtURL_toURL_error(&src, &dst)
        .map_err(|e| e.localizedDescription().to_string())
}

fn file_name(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

/// Legt alle Dateien eines Vorgangs zurück an ihren alten Ort.
pub fn restore_batch(batch: u64) -> Vec<Restored> {
    // `partition` teilt in zwei Listen: (dieser Vorgang, alles andere).
    let (restore, keep): (Vec<Entry>, Vec<Entry>) = read_log().into_iter().partition(|e| e.batch == batch);
    let mut results = Vec::new();
    let mut failed = Vec::new(); // bleiben im Protokoll, damit man es später nochmal versuchen kann

    for e in restore {
        let name = file_name(&e.original);
        let result = if std::fs::symlink_metadata(&e.original).is_ok() {
            Err(tr("an der alten Stelle liegt schon etwas mit diesem Namen – bitte von Hand aus dem Papierkorb holen", "something with this name is already at the old location – please restore it from the Trash by hand"))
        } else if std::fs::symlink_metadata(&e.trashed).is_err() {
            Err(tr("ist nicht mehr im Papierkorb (vielleicht wurde er geleert)", "is no longer in the Trash (maybe it was emptied)"))
        } else {
            if let Some(parent) = e.original.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            move_item(&e.trashed, &e.original).map_err(|err| t!("konnte nicht zurückgelegt werden: {err}", "could not be put back: {err}"))
        };
        match result {
            Ok(()) => results.push(Restored { name, ok: true, message: t!("ist wieder da: {}", "is back: {}", e.original.display()) }),
            Err(message) => {
                // Nur wiederholbare Fehler aufheben; „nicht mehr im Papierkorb“ ist endgültig.
                if std::fs::symlink_metadata(&e.trashed).is_ok() {
                    failed.push(e);
                }
                results.push(Restored { name, ok: false, message });
            }
        }
    }
    let mut remaining = keep;
    remaining.extend(failed);
    remaining.sort_by_key(|e| e.batch);
    let _ = write_log(&remaining);
    results
}

/// Die Vorgänge für die Papierkorb-Liste, neueste zuerst.
pub fn list_batches() -> Vec<Batch> {
    let mut batches: Vec<Batch> = Vec::new();
    for e in read_log() {
        let available = std::fs::symlink_metadata(&e.trashed).is_ok();
        let item = Item {
            name: file_name(&e.original),
            folder: e.original.parent().map(|p| p.display().to_string()).unwrap_or_default(),
            is_dir: e.trashed.is_dir(),
            available,
        };
        // `last_mut()` = das zuletzt hinzugefügte Element zum Verändern
        match batches.last_mut() {
            Some(b) if b.batch == e.batch => b.items.push(item),
            _ => batches.push(Batch { batch: e.batch, items: vec![item] }),
        }
    }
    batches.reverse();
    batches.truncate(50);
    batches
}

pub fn last_batch() -> Option<u64> {
    read_log().last().map(|e| e.batch)
}

// ── Protokoll-Datei ──

fn log_path() -> PathBuf {
    // Für Tests lässt sich der Ort umbiegen.
    if let Some(p) = std::env::var_os("KOMPASS_TRASH_LOG") {
        return PathBuf::from(p);
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    home.join("Library/Application Support/Kompass/papierkorb.jsonl")
}

pub fn append_log(entries: &[Entry]) -> std::io::Result<()> {
    let path = log_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    for e in entries {
        writeln!(f, "{}", serde_json::to_string(e)?)?;
    }
    Ok(())
}

fn read_log() -> Vec<Entry> {
    std::fs::read_to_string(log_path())
        .unwrap_or_default()
        .lines()
        // `filter_map` überspringt Zeilen, die sich nicht lesen lassen.
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

fn write_log(entries: &[Entry]) -> std::io::Result<()> {
    let text: String = entries
        .iter()
        .filter_map(|e| serde_json::to_string(e).ok())
        .map(|l| l + "\n")
        .collect();
    std::fs::write(log_path(), text)
}

// ── Befehle für das Fenster ──

#[tauri::command]
pub fn trash_list() -> Vec<Batch> {
    list_batches()
}

#[tauri::command]
pub fn trash_restore(batch: u64) -> Vec<Restored> {
    restore_batch(batch)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Echter Papierkorb-Durchlauf: rein, Liste prüfen, zurück.
    /// Legt nur eine kleine Testdatei an und holt sie sofort zurück.
    #[test]
    fn papierkorb_hin_und_zurueck() {
        let dir = std::env::temp_dir().join(format!("kompass-trashlog-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // Eigenes Protokoll, damit dein echtes unberührt bleibt.
        std::env::set_var("KOMPASS_TRASH_LOG", dir.join("log.jsonl"));
        let file = dir.join("kompass-selbsttest.txt");
        std::fs::write(&file, "bin gleich wieder da").unwrap();

        let trashed = move_to_trash(&file).unwrap();
        assert!(!file.exists());
        let batch = now_ms();
        append_log(&[Entry { batch, original: file.clone(), trashed }]).unwrap();

        let list = list_batches();
        assert_eq!(list[0].items[0].name, "kompass-selbsttest.txt");
        assert!(list[0].items[0].available);

        let res = restore_batch(batch);
        assert!(res[0].ok, "{}", res[0].message);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "bin gleich wieder da");
        assert!(list_batches().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
