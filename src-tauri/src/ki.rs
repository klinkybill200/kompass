//! Optionale KI-Übersetzung über die Claude API.
//!
//! Nur aktiv, wenn ein API-Schlüssel hinterlegt ist – im macOS-Schlüsselbund
//! (Einstellungen in Kompass) oder als Umgebungsvariable ANTHROPIC_API_KEY.
//! Ohne Schlüssel funktioniert Kompass genauso, nur ohne KI-Rückfall.

use crate::i18n::{is_en, tr};
use crate::nl::Suggestion;
use serde::Serialize;
use serde_json::{json, Value};
use std::path::Path;
use std::time::Duration;

const MODEL: &str = "claude-opus-5-5";
const KEYCHAIN_SERVICE: &str = "net.underdock.kompass";
const KEYCHAIN_ACCOUNT: &str = "anthropic-api-key";

/// Woher der Schlüssel kommt (für die Anzeige in den Einstellungen).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KiStatus {
    pub configured: bool,
    /// "keychain", "env" oder "none"
    pub source: &'static str,
    pub model: &'static str,
}

fn keychain() -> Option<keyring::Entry> {
    keyring::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT).ok()
}

/// Liefert den Schlüssel: zuerst Schlüsselbund, sonst Umgebungsvariable.
fn api_key() -> Option<(String, &'static str)> {
    if let Some(k) = keychain().and_then(|e| e.get_password().ok()).filter(|k| !k.trim().is_empty()) {
        return Some((k, "keychain"));
    }
    std::env::var("ANTHROPIC_API_KEY").ok().filter(|k| !k.trim().is_empty()).map(|k| (k, "env"))
}

pub fn available() -> bool {
    api_key().is_some()
}

#[tauri::command]
pub fn ki_status() -> KiStatus {
    let source = api_key().map(|(_, s)| s).unwrap_or("none");
    KiStatus { configured: source != "none", source, model: MODEL }
}

/// Schlüssel speichern (der Nutzer gibt ihn selbst in den Einstellungen ein).
#[tauri::command]
pub fn ki_set_key(key: String) -> Result<KiStatus, String> {
    let key = key.trim();
    if !key.starts_with("sk-ant-") {
        return Err(tr("Das sieht nicht nach einem Anthropic-API-Schlüssel aus (beginnt mit „sk-ant-“).", "That does not look like an Anthropic API key (starts with “sk-ant-”)."));
    }
    keychain()
        .ok_or_else(|| tr("Schlüsselbund nicht erreichbar", "Keychain not reachable"))?
        .set_password(key)
        .map_err(|e| t!("Konnte nicht im Schlüsselbund speichern: {e}", "Could not save to the keychain: {e}"))?;
    Ok(ki_status())
}

#[tauri::command]
pub fn ki_clear_key() -> KiStatus {
    if let Some(e) = keychain() {
        let _ = e.delete_credential();
    }
    ki_status()
}

/// Testlauf aus den Einstellungen. `async` + eigener Thread, damit das Fenster nicht hängt.
#[tauri::command]
pub async fn ki_test() -> Result<Suggestion, String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
    tauri::async_runtime::spawn_blocking(move || translate(&tr("zeige mir die fünf neuesten Dateien", "show me the five newest files"), Path::new(&home)))
        .await
        .map_err(|e| e.to_string())?
}

const SYSTEM: &str = "Du bist der Übersetzer in Kompass, einem einsteigerfreundlichen Terminal für macOS (zsh). \
Der Nutzer beschreibt auf Deutsch, was er tun möchte. Antworte mit genau EINEM zsh-Befehl (Pipes und && sind erlaubt), \
der auf einem aktuellen macOS ohne Zusatzsoftware läuft (BSD-Werkzeuge, nicht GNU). \
Bevorzuge einfache, gut lesbare Befehle, die ein Einsteiger nachvollziehen kann. Verwende Dateinamen aus der Ordnerliste, wenn sie passen, \
und setze Namen mit Leerzeichen in einfache Anführungszeichen. Nutze kein sudo, außer es geht wirklich nicht anders. \
„explanation“ ist ein kurzer deutscher Satz (Du-Form), was der Befehl tut. \
Wenn der Wunsch kein Terminal-Befehl ist oder unklar bleibt, setze understood auf false, command auf einen leeren Text \
und erkläre in explanation kurz, was fehlt.";

/// Fragt Claude nach einem Befehl. Blockiert (bis zu ~60 s) – nur in einem eigenen Thread aufrufen.
pub fn translate(query: &str, cwd: &Path) -> Result<Suggestion, String> {
    let (key, _) = api_key().ok_or_else(|| tr("Kein API-Schlüssel hinterlegt", "No API key stored"))?;

    // Ein paar Namen aus dem Ordner mitschicken, damit die KI echte Dateien kennt.
    let mut entries: Vec<String> = std::fs::read_dir(cwd)
        .map(|d| d.flatten().take(60).map(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            if e.path().is_dir() { format!("{n}/") } else { n }
        }).collect())
        .unwrap_or_default();
    entries.sort();
    let context = format!("Aktueller Ordner: {}\nInhalt (Auszug): {}\n\nWunsch: {query}", cwd.display(), entries.join(", "));

    let body = json!({
        "model": MODEL,
        "max_tokens": 4000,
        "system": if is_en() {
            format!("{SYSTEM}\n\nThe user writes in English: understand English requests and write „explanation“ in English.")
        } else {
            SYSTEM.to_string()
        },
        "fallbacks": "default",
        "output_config": {
            "effort": "low",
            "format": {
                "type": "json_schema",
                "schema": {
                    "type": "object",
                    "properties": {
                        "understood": { "type": "boolean" },
                        "command": { "type": "string" },
                        "explanation": { "type": "string" }
                    },
                    "required": ["understood", "command", "explanation"],
                    "additionalProperties": false
                }
            }
        },
        "messages": [{ "role": "user", "content": context }]
    });

    let response = ureq::post("https://api.anthropic.com/v1/messages")
        .set("x-api-key", &key)
        .set("anthropic-version", "2023-06-01")
        .set("anthropic-beta", "server-side-fallback-2026-07-01")
        .set("content-type", "application/json")
        .timeout(Duration::from_secs(60))
        .send_json(body);

    let value: Value = match response {
        Ok(r) => r.into_json().map_err(|e| t!("Antwort unlesbar: {e}", "Unreadable response: {e}"))?,
        Err(ureq::Error::Status(code, r)) => {
            let detail = r.into_json::<Value>().ok().and_then(|v| v["error"]["message"].as_str().map(String::from)).unwrap_or_default();
            return Err(match code {
                401 => tr("Der API-Schlüssel wurde abgelehnt – bitte in den Einstellungen prüfen.", "The API key was rejected – please check it in the settings."),
                429 => tr("Zu viele Anfragen – bitte kurz warten.", "Too many requests – please wait a moment."),
                c if c >= 500 => tr("Der KI-Dienst ist gerade nicht erreichbar.", "The AI service is currently unavailable."),
                c => t!("KI-Fehler {c}: {detail}", "AI error {c}: {detail}"),
            });
        }
        Err(e) => return Err(t!("Keine Verbindung zum KI-Dienst: {e}", "No connection to the AI service: {e}")),
    };

    if value["stop_reason"] == "refusal" {
        return Err(tr("Die KI möchte diese Anfrage nicht übersetzen.", "The AI declined to translate this request."));
    }
    // Den Text-Block mit dem JSON finden (Denk-Blöcke überspringen).
    let text = value["content"]
        .as_array()
        .and_then(|blocks| blocks.iter().find(|b| b["type"] == "text"))
        .and_then(|b| b["text"].as_str())
        .ok_or_else(|| tr("Leere Antwort von der KI", "Empty response from the AI"))?;
    let parsed: Value = serde_json::from_str(text).map_err(|e| t!("KI-Antwort unlesbar: {e}", "Unreadable AI response: {e}"))?;
    let command = parsed["command"].as_str().unwrap_or("").trim().to_string();
    let explanation = parsed["explanation"].as_str().unwrap_or("").to_string();
    if parsed["understood"] != true || command.is_empty() {
        return Err(if explanation.is_empty() { tr("Die KI hat den Wunsch nicht verstanden.", "The AI did not understand the request.") } else { explanation });
    }
    // Sicherheitsnetz: nur eine Zeile übernehmen.
    let command = command.lines().next().unwrap_or("").to_string();
    Ok(Suggestion { command, text: explanation, needs_input: false, source: "ki" })
}
