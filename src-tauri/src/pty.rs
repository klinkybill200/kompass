//! Startet eine echte zsh in einem Pseudo-Terminal (PTY) und reicht
//! Ein- und Ausgabe zwischen Shell und Frontend (xterm.js) durch.
//!
//! Außerdem die Brücke zum Wächter: Drückt man Enter, schickt die zsh die
//! Zeile unsichtbar an uns (siehe shell/kompass.zsh). Wir prüfen sie mit
//! `guard::check` und antworten: ausführen, umgeschrieben ausführen oder abbrechen.

// `use` holt Namen aus anderen Bibliotheken in den Gültigkeitsbereich.
use crate::errors::{self, Help};
use crate::explain::{self, Explanation};
use crate::guard::{self, Level, Verdict};
use crate::i18n::tr;
use crate::ki;
use crate::nl::{self, Suggestion};
use crate::trashlog;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;

/// Nachrichten, die das Backend an das Frontend schickt.
/// `#[derive(...)]` lässt den Compiler Code automatisch erzeugen –
/// hier: Umwandlung in JSON (`Serialize`) und Kopieren (`Clone`).
/// `tag = "kind"` erzeugt JSON wie `{ "kind": "output", "data": "..." }`.
#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ServerMsg {
    /// Text, den die Shell ausgegeben hat.
    Output { data: String },
    /// Der Wächter hat eine Zeile bewertet. Bei `needs_confirm` wartet die
    /// Shell, bis das Frontend `guard_answer` aufruft.
    #[serde(rename_all = "camelCase")]
    Verdict { id: u64, command: String, verdict: Verdict, needs_confirm: bool },
    /// Live-Erklärung der Zeile, die gerade getippt wird (leer = nichts getippt).
    /// `nl` ist gesetzt, wenn die Zeile nach Klarsprache aussieht.
    Live { line: String, explanation: Option<Explanation>, nl: Option<NlLive> },
    /// Klarsprache wurde in einen Befehl übersetzt (steht jetzt in der Eingabezeile).
    Translated { query: String, suggestion: Suggestion },
    /// Die KI überlegt gerade (Esc bricht ab).
    Thinking { id: u64, query: String },
    /// Klarsprache nicht verstanden.
    #[serde(rename_all = "camelCase")]
    NotUnderstood { query: String, reason: String, ki_available: bool },
    /// Ein Befehl ist fertig. `exit_code` 0 = hat geklappt.
    #[serde(rename_all = "camelCase")]
    Done { command: String, exit_code: i32, cwd: String },
    /// Übersetzte Fehlermeldung zum letzten Befehl.
    Help { command: String, help: Help },
    /// `kompass-trash` hat Dateien in den Papierkorb gelegt.
    Trashed { batch: u64, items: Vec<trashlog::Item> },
    /// Die Shell wurde beendet (z. B. mit `exit`).
    Exit,
}

/// Live-Vorschau für Klarsprache.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NlLive {
    pub suggestion: Option<Suggestion>,
    pub ki_available: bool,
}

/// Eine laufende Shell-Sitzung.
/// `Box<dyn Trait>` = „irgendein Objekt, das diese Fähigkeit hat“ (wie ein Interface).
/// `Send` bedeutet: darf zwischen Threads weitergereicht werden.
struct Session {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
    /// Wartet diese Shell gerade auf eine Antwort (Dialog oder KI)?
    pending: Arc<Mutex<Option<Pending>>>,
}

/// Eine Zeile, die auf eine Antwort wartet – vom Bestätigungsdialog oder von der KI.
struct Pending {
    id: u64,
    rewritten: Option<String>, // Papierkorb-Variante, falls vorhanden
}

/// Der gemeinsame Zustand der App.
/// `Arc<Mutex<...>>`: `Arc` erlaubt mehrere Besitzer (mehrere Threads),
/// `Mutex` sorgt dafür, dass immer nur einer gleichzeitig darauf zugreift.
/// `Option` heißt: kann leer sein (`None`), z. B. solange noch keine Shell läuft.
/// Jeder Tab hat seine eigene Shell; der Schlüssel ist die Tab-Nummer (`sid`).
/// `HashMap` = Nachschlagetabelle Nummer → Sitzung.
#[derive(Default)]
pub struct PtyState {
    sessions: Mutex<HashMap<u32, Session>>,
}

/// Fortlaufende Nummer für Wächter-Anfragen. `static` = existiert genau einmal.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Startet eine neue zsh. Wird vom Frontend beim Laden aufgerufen.
/// `#[tauri::command]` macht die Funktion für `invoke("pty_spawn")` verfügbar.
/// `Result<(), String>`: entweder Erfolg (`Ok(())`) oder Fehlertext (`Err(...)`).
#[tauri::command]
pub fn pty_spawn(
    state: tauri::State<'_, PtyState>,
    on_event: Channel<ServerMsg>,
    sid: u32,
    cols: u16,
    rows: u16,
    cwd: Option<String>,
) -> Result<(), String> {
    // Eine evtl. noch laufende alte Shell dieses Tabs beenden (z. B. nach Neuladen der Seite).
    // `if let Some(x) = ...` = „falls etwas drin ist, nimm es als x“.
    if let Some(mut old) = state.sessions.lock().unwrap().remove(&sid) {
        let _ = old.child.kill(); // `let _ =` ignoriert bewusst das Ergebnis.
    }

    let zdotdir = install_shell_files().map_err(|e| format!("Shell-Integration fehlt: {e}"))?;

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
        // `map_err` wandelt den Fehler in Text um, `?` gibt ihn sofort zurück.
        .map_err(|e| format!("PTY konnte nicht geöffnet werden: {e}"))?;

    let mut cmd = shell_command(&zdotdir);
    // Neuer Tab startet im Ordner des aktuellen Tabs (wie bei Terminal.app).
    if let Some(dir) = cwd.filter(|d| Path::new(d).is_dir()) {
        cmd.cwd(dir);
    }

    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("zsh konnte nicht gestartet werden: {e}"))?;
    // `eprintln!` schreibt ins Log (Konsole) – praktisch zur Fehlersuche.
    eprintln!("[kompass] zsh gestartet (Tab {sid}, pid {:?}, {cols}x{rows})", child.process_id());
    // Die „Slave“-Seite gehört jetzt der Shell; wir brauchen sie nicht mehr.
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = Arc::new(Mutex::new(pair.master.take_writer().map_err(|e| e.to_string())?));

    let pending = Arc::new(Mutex::new(None));
    state.sessions.lock().unwrap().insert(sid, Session { writer: writer.clone(), master: pair.master, child, pending: pending.clone() });

    // Eigener Thread, der dauerhaft die Ausgabe der Shell liest.
    // `move` überträgt die benötigten Variablen in den Thread;
    // `.clone()` auf einem Arc erzeugt nur einen weiteren Verweis, keine Kopie.
    let mut router = Router { writer, pending, on_event, capture: None };
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        let mut scanner = OscScanner::default();
        let mut decoder = Utf8Decoder::default();
        loop {
            match reader.read(&mut buf) {
                // 0 Bytes oder Fehler → Shell ist beendet.
                Ok(0) | Err(_) => {
                    let _ = router.on_event.send(ServerMsg::Exit);
                    break;
                }
                Ok(n) => {
                    // Unsichtbare Kompass-Nachrichten heraussieben, Rest ist normale Ausgabe.
                    let (visible, messages) = scanner.feed(&buf[..n]);
                    router.record(&visible);
                    let text = decoder.decode(&visible);
                    if !text.is_empty() {
                        let _ = router.on_event.send(ServerMsg::Output { data: text });
                    }
                    for (kind, payload) in messages {
                        router.handle(kind, &payload);
                    }
                }
            }
        }
    });

    Ok(())
}

/// Mitschnitt der Ausgabe des gerade laufenden Befehls (für den Fehler-Übersetzer).
struct Capture {
    command: String,
    output: Vec<u8>,
}

const CAPTURE_LIMIT: usize = 32 * 1024; // nur die letzten 32 KB aufheben

/// Verteilt die Nachrichten der Shell. Lebt im Lese-Thread.
struct Router {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pending: Arc<Mutex<Option<Pending>>>,
    on_event: Channel<ServerMsg>,
    capture: Option<Capture>,
}

impl Router {
    /// Sichtbare Ausgabe mitschneiden, solange ein Befehl läuft.
    fn record(&mut self, visible: &[u8]) {
        if let Some(cap) = self.capture.as_mut() {
            cap.output.extend_from_slice(visible);
            if cap.output.len() > CAPTURE_LIMIT {
                let cut = cap.output.len() - CAPTURE_LIMIT;
                cap.output.drain(..cut);
            }
        }
    }

    /// Nachrichtenarten (Zahl hinter `ESC ] 777`):
    /// 0 = Prüfanfrage vor dem Ausführen, 1 = Live-Eingabe,
    /// 2 = Befehl fertig, 3 = Dateien im Papierkorb.
    fn handle(&mut self, kind: u8, payload: &[u8]) {
        match kind {
            0 => self.check(payload),
            1 => self.live(payload),
            2 => self.done(payload),
            3 => self.trashed(payload),
            _ => {}
        }
    }

    /// Prüfanfrage: `<base64 Befehl>;<base64 Ordner>;<bekannt 0/1>`.
    fn check(&mut self, payload: &[u8]) {
        let decode = |part: &[u8]| -> Option<String> { String::from_utf8(B64.decode(part).ok()?).ok() };
        let mut parts = payload.splitn(3, |b| *b == b';');
        let command = parts.next().and_then(decode);
        let cwd = parts.next().and_then(decode);
        // Fehlt die Angabe (ältere Shell-Integration), gilt der Befehl als bekannt.
        let known = parts.next().map(|k| k != b"0").unwrap_or(true);

        // `let (Some(a), Some(b)) = ... else { ... }`: beide müssen da sein, sonst Notausgang.
        let (Some(command), Some(cwd)) = (command, cwd) else {
            // Unlesbare Anfrage: lieber ausführen als die Shell blockieren.
            send_reply(&self.writer, "r.");
            return;
        };

        // Klarsprache? Dann übersetzen statt ausführen.
        if let Some(query) = nl::detect(&command, known) {
            if self.translate(&command, query, &cwd) {
                return;
            }
        } else if known && nl::looks_like_sentence(&command) {
            // „find all pdf files“: beginnt mit einem echten Befehl, ist aber ein Satz.
            // Nur übersetzen, wenn das Regelwerk ihn sicher versteht – sonst normal ausführen.
            if let Some(suggestion) = nl::translate(&command, Path::new(&cwd)) {
                send_reply(&self.writer, &format!("e{}.", B64.encode(&suggestion.command)));
                let _ = self.on_event.send(ServerMsg::Translated { query: command, suggestion });
                return;
            }
        }

        let verdict = guard::check(&command, Path::new(&cwd));
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let needs_confirm = verdict.level == Level::Danger;
        eprintln!("[kompass] Wächter: {:?} ← {command}", verdict.level);

        if needs_confirm {
            // Shell wartet jetzt; die Antwort kommt über `guard_answer`.
            let rewritten = verdict.trash.as_ref().map(|t| t.rewritten.clone());
            *self.pending.lock().unwrap() = Some(Pending { id, rewritten });
        } else {
            send_reply(&self.writer, "r.");
        }
        self.capture = Some(Capture { command: command.clone(), output: Vec::new() });
        let _ = self.on_event.send(ServerMsg::Verdict { id, command, verdict, needs_confirm });
    }

    /// Klarsprache übersetzen. Gibt `false` zurück, wenn es doch als normaler Befehl laufen soll.
    fn translate(&mut self, line: &str, query: String, cwd: &str) -> bool {
        let explicit = line.trim_start().starts_with('?');
        let single_word = !query.contains(char::is_whitespace);

        if let Some(suggestion) = nl::translate(&query, Path::new(cwd)) {
            // Befehl in die Eingabezeile setzen – ausgeführt wird erst beim nächsten Enter.
            send_reply(&self.writer, &format!("e{}.", B64.encode(&suggestion.command)));
            let _ = self.on_event.send(ServerMsg::Translated { query, suggestion });
            return true;
        }
        // Ein einzelnes unbekanntes Wort ist eher ein Tippfehler als Klarsprache
        // → normal ausführen, dann greift die Fehlerhilfe („Meintest du …?“).
        if single_word && !explicit {
            return false;
        }
        if ki::available() {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            *self.pending.lock().unwrap() = Some(Pending { id, rewritten: None });
            let _ = self.on_event.send(ServerMsg::Thinking { id, query: query.clone() });
            // Die KI-Anfrage dauert ein paar Sekunden → eigener Thread, damit die Ausgabe weiterläuft.
            let (writer, pending, on_event, cwd) = (self.writer.clone(), self.pending.clone(), self.on_event.clone(), cwd.to_string());
            std::thread::spawn(move || {
                let result = ki::translate(&query, Path::new(&cwd));
                // Nur antworten, wenn niemand zwischendurch mit Esc abgebrochen hat.
                let still_waiting = {
                    let mut p = pending.lock().unwrap();
                    if p.as_ref().map(|p| p.id) == Some(id) { p.take(); true } else { false }
                };
                if !still_waiting {
                    return;
                }
                match result {
                    Ok(suggestion) => {
                        send_reply(&writer, &format!("e{}.", B64.encode(&suggestion.command)));
                        let _ = on_event.send(ServerMsg::Translated { query, suggestion });
                    }
                    Err(reason) => {
                        send_reply(&writer, &not_understood_reply());
                        let _ = on_event.send(ServerMsg::NotUnderstood { query, reason, ki_available: true });
                    }
                }
            });
            return true;
        }
        send_reply(&self.writer, &not_understood_reply());
        let _ = self.on_event.send(ServerMsg::NotUnderstood {
            query,
            reason: tr("Das kennt Kompass noch nicht.", "Kompass does not know that yet."),
            ki_available: false,
        });
        true
    }

    /// Live-Eingabe: `<Ordner> US <bekannt 0/1> US <Zeile>` (US = Trennzeichen 0x1F).
    fn live(&mut self, payload: &[u8]) {
        let text = String::from_utf8_lossy(payload);
        let mut parts = text.splitn(3, '\x1f');
        let (Some(cwd), Some(known), Some(line)) = (parts.next(), parts.next(), parts.next()) else { return };
        let mut nl_live = None;
        let mut explanation = None;
        if !line.trim().is_empty() {
            match nl::detect(line, known != "0") {
                Some(query) => {
                    let suggestion = nl::translate(&query, Path::new(cwd));
                    // Einzelne unbekannte Wörter ohne Treffer: normal erklären (Tippfehler-Hinweis).
                    if suggestion.is_some() || query.contains(char::is_whitespace) || line.trim_start().starts_with('?') {
                        nl_live = Some(NlLive { suggestion, ki_available: ki::available() });
                    } else {
                        explanation = Some(explain::explain(line, Path::new(cwd)));
                    }
                }
                None => {
                    let sentence = (known != "0" && nl::looks_like_sentence(line)).then(|| nl::translate(line, Path::new(cwd))).flatten();
                    match sentence {
                        Some(s) => nl_live = Some(NlLive { suggestion: Some(s), ki_available: ki::available() }),
                        None => explanation = Some(explain::explain(line, Path::new(cwd))),
                    }
                }
            }
        }
        let _ = self.on_event.send(ServerMsg::Live { line: line.to_string(), explanation, nl: nl_live });
    }

    /// Befehl fertig: `<Rückgabewert> US <Ordner>`.
    fn done(&mut self, payload: &[u8]) {
        // `take()` holt den Mitschnitt heraus und lässt `None` zurück.
        let Some(cap) = self.capture.take() else { return }; // z. B. erster Prompt
        let text = String::from_utf8_lossy(payload);
        let (code, cwd) = text.split_once('\x1f').unwrap_or((&text, ""));
        let exit_code: i32 = code.trim().parse().unwrap_or(0);
        let output = String::from_utf8_lossy(&cap.output);
        if let Some(help) = errors::analyze(&cap.command, exit_code, &output, Path::new(cwd)) {
            let _ = self.on_event.send(ServerMsg::Help { command: cap.command.clone(), help });
        }
        let _ = self.on_event.send(ServerMsg::Done { command: cap.command, exit_code, cwd: cwd.to_string() });
    }

    /// Papierkorb-Meldung von `kompass-trash`: `<Vorgangsnummer>`.
    fn trashed(&mut self, payload: &[u8]) {
        let Ok(batch) = String::from_utf8_lossy(payload).trim().parse::<u64>() else { return };
        let items = trashlog::list_batches().into_iter().find(|b| b.batch == batch).map(|b| b.items).unwrap_or_default();
        let _ = self.on_event.send(ServerMsg::Trashed { batch, items });
    }
}

/// Antwort „nicht verstanden“ samt Meldung für das Terminal.
fn not_understood_reply() -> String {
    format!("n{}.", B64.encode(tr(
        "🤔  Das habe ich nicht verstanden – formulier es anders oder tippe einen Befehl.",
        "🤔  I did not understand that – rephrase it or type a command.",
    )))
}

fn send_reply(writer: &Arc<Mutex<Box<dyn Write + Send>>>, reply: &str) {
    let mut w = writer.lock().unwrap();
    let _ = w.write_all(reply.as_bytes());
    let _ = w.flush();
}

/// Antwort aus dem Bestätigungsdialog: "trash", "run" oder "cancel".
/// Auch zum Abbrechen einer laufenden KI-Anfrage ("cancel").
#[tauri::command]
pub fn guard_answer(state: tauri::State<'_, PtyState>, sid: u32, id: u64, choice: String) -> Result<(), String> {
    let sessions = state.sessions.lock().unwrap();
    let session = sessions.get(&sid).ok_or("Keine Shell aktiv")?;
    let mut pending = session.pending.lock().unwrap();
    // Nur antworten, wenn die Nummer zur wartenden Anfrage passt.
    let p = match pending.take() {
        Some(p) if p.id == id => p,
        other => {
            *pending = other; // gehört nicht zu uns → zurücklegen
            return Err("Diese Anfrage ist nicht mehr aktuell".into());
        }
    };
    let reply = match (choice.as_str(), p.rewritten) {
        ("trash", Some(cmd)) => format!("r{}.", B64.encode(cmd)),
        ("run", _) => "r.".to_string(),
        _ => format!("c{}.", B64.encode(tr("↩  Abgebrochen – es ist nichts passiert.", "↩  Cancelled – nothing happened."))),
    };
    send_reply(&session.writer, &reply);
    Ok(())
}

/// Tastatureingaben aus xterm.js an die Shell weiterreichen.
#[tauri::command]
pub fn pty_write(state: tauri::State<'_, PtyState>, sid: u32, data: String) -> Result<(), String> {
    let sessions = state.sessions.lock().unwrap();
    // `get` sucht in der Tabelle; `ok_or` macht aus „nicht gefunden“ einen Fehler.
    let session = sessions.get(&sid).ok_or("Keine Shell aktiv")?;
    // Während Dialog oder KI laufen, wartet die zsh auf unsere Antwort –
    // Tastendrücke würden die Antwort verfälschen, also verwerfen wir sie.
    if session.pending.lock().unwrap().is_some() {
        return Ok(());
    }
    let mut w = session.writer.lock().unwrap();
    w.write_all(data.as_bytes()).map_err(|e| e.to_string())?;
    w.flush().map_err(|e| e.to_string())
}

/// Fenstergröße geändert → der Shell die neue Spalten-/Zeilenzahl mitteilen.
#[tauri::command]
pub fn pty_resize(state: tauri::State<'_, PtyState>, sid: u32, cols: u16, rows: u16) -> Result<(), String> {
    if let Some(session) = state.sessions.lock().unwrap().get(&sid) {
        session
            .master
            .resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Tab geschlossen → Shell beenden.
#[tauri::command]
pub fn pty_close(state: tauri::State<'_, PtyState>, sid: u32) {
    if let Some(mut s) = state.sessions.lock().unwrap().remove(&sid) {
        let _ = s.child.kill();
    }
}

/// Baut den Startbefehl für die zsh zusammen.
/// `-> CommandBuilder` gibt an, was die Funktion zurückliefert.
fn shell_command(zdotdir: &Path) -> CommandBuilder {
    // zsh als Login-Shell starten – genau wie die Terminal-App von macOS.
    let mut cmd = CommandBuilder::new("/bin/zsh");
    cmd.arg("-l");
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    cmd.env("TERM_PROGRAM", "Kompass");
    // Über ZDOTDIR lädt die zsh zuerst unsere Dateien (die dann deine eigenen laden).
    cmd.env("ZDOTDIR", zdotdir);
    if let Some(user_zdotdir) = std::env::var_os("ZDOTDIR") {
        cmd.env("KOMPASS_USER_ZDOTDIR", user_zdotdir);
    }
    // Pfad zu diesem Programm – `kompass-trash` ruft es mit `--kompass-trash` auf.
    if let Ok(exe) = std::env::current_exe() {
        cmd.env("KOMPASS_BIN", exe);
    }
    // Aus dem Finder gestartet fehlt oft LANG → Umlaute wären kaputt.
    if std::env::var("LANG").is_err() {
        cmd.env("LANG", "de_DE.UTF-8");
    }
    if let Some(home) = std::env::var_os("HOME") {
        cmd.cwd(home);
    }
    cmd
}

/// Schreibt die zsh-Integrationsdateien in einen temporären Ordner.
/// `include_str!` bettet die Dateien schon beim Kompilieren ins Programm ein.
fn install_shell_files() -> std::io::Result<PathBuf> {
    let dir = std::env::temp_dir().join("kompass-shell");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(".zshenv"), include_str!("../shell/zshenv"))?;
    std::fs::write(dir.join(".zprofile"), include_str!("../shell/zprofile"))?;
    std::fs::write(dir.join(".zshrc"), include_str!("../shell/zshrc"))?;
    std::fs::write(dir.join("kompass.zsh"), include_str!("../shell/kompass.zsh"))?;
    Ok(dir)
}

/// Findet in der Ausgabe die unsichtbaren Kompass-Nachrichten `ESC ] 777<Art> ; … BEL`.
/// Eine Nachricht kann auf zwei Lesevorgänge verteilt ankommen – deshalb der Puffer.
#[derive(Default)]
struct OscScanner {
    buf: Vec<u8>,
}

const MARKER: &[u8] = b"\x1b]777";
const MAX_MESSAGE: usize = 64 * 1024; // Schutz, falls das Ende nie kommt

impl OscScanner {
    /// Gibt (sichtbare Ausgabe, gefundene Nachrichten als (Art, Inhalt)) zurück.
    fn feed(&mut self, chunk: &[u8]) -> (Vec<u8>, Vec<(u8, Vec<u8>)>) {
        self.buf.extend_from_slice(chunk);
        let mut visible = Vec::new();
        let mut messages = Vec::new();
        loop {
            match find(&self.buf, MARKER) {
                Some(start) => {
                    visible.extend_from_slice(&self.buf[..start]);
                    self.buf.drain(..start);
                    let body = MARKER.len();
                    // Es fehlen noch Art und Semikolon → auf mehr Daten warten.
                    if self.buf.len() < body + 2 {
                        break;
                    }
                    let kind = self.buf[body];
                    if !kind.is_ascii_digit() || self.buf[body + 1] != b';' {
                        // Sieht nur ähnlich aus, gehört nicht uns → normal anzeigen.
                        visible.extend_from_slice(&self.buf[..body]);
                        self.buf.drain(..body);
                        continue;
                    }
                    match self.buf[body + 2..].iter().position(|&b| b == 0x07) {
                        Some(len) => {
                            messages.push((kind - b'0', self.buf[body + 2..body + 2 + len].to_vec()));
                            self.buf.drain(..body + 2 + len + 1);
                        }
                        None if self.buf.len() > MAX_MESSAGE => {
                            visible.extend_from_slice(&self.buf);
                            self.buf.clear();
                            break;
                        }
                        None => break, // Ende fehlt noch
                    }
                }
                None => {
                    // Das Ende könnte der Anfang eines Markers sein (z. B. nur `ESC ]77`).
                    let keep = (1..MARKER.len())
                        .rev()
                        .find(|&k| self.buf.len() >= k && self.buf.ends_with(&MARKER[..k]))
                        .unwrap_or(0);
                    let cut = self.buf.len() - keep;
                    visible.extend_from_slice(&self.buf[..cut]);
                    self.buf.drain(..cut);
                    break;
                }
            }
        }
        (visible, messages)
    }
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// Wandelt Bytes in Text um. Ein Umlaut besteht aus 2 Bytes – kommt nur
/// das erste an, merken wir es uns, bis das zweite eintrifft.
#[derive(Default)]
struct Utf8Decoder {
    rest: Vec<u8>, // `Vec` = wachsende Liste
}

impl Utf8Decoder {
    // `&mut self` = diese Methode darf das Objekt verändern.
    fn decode(&mut self, bytes: &[u8]) -> String {
        self.rest.extend_from_slice(bytes);
        match std::str::from_utf8(&self.rest) {
            Ok(s) => {
                let out = s.to_string();
                self.rest.clear();
                out
            }
            Err(e) => {
                let valid = e.valid_up_to();
                // `error_len() == None` → Zeichen ist nur unvollständig, später weiterlesen.
                let keep_from = if e.error_len().is_none() { valid } else { self.rest.len() };
                let out = String::from_utf8_lossy(&self.rest[..keep_from]).into_owned();
                self.rest.drain(..keep_from);
                out
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anfrage_ueber_zwei_pakete_verteilt() {
        let mut s = OscScanner::default();
        let (v1, r1) = s.feed(b"hallo \x1b]77");
        assert_eq!(v1, b"hallo ");
        assert!(r1.is_empty());
        let (v2, r2) = s.feed(b"70;bHM=;Lw==\x07 welt");
        assert_eq!(v2, b" welt");
        assert_eq!(r2, vec![(0, b"bHM=;Lw==".to_vec())]);
        // Fremde OSC-Sequenzen (z. B. Fenstertitel) bleiben sichtbar.
        let (v3, r3) = s.feed(b"\x1b]7;file://x\x07\x1b]7772;0\x1f/tmp\x07");
        assert_eq!(v3, b"\x1b]7;file://x\x07");
        assert_eq!(r3, vec![(2, b"0\x1f/tmp".to_vec())]);
    }

    #[test]
    fn umlaut_ueber_zwei_pakete_verteilt() {
        let mut d = Utf8Decoder::default();
        let bytes = "Grüße".as_bytes();
        let a = d.decode(&bytes[..3]); // "Gr" + halbes ü
        let b = d.decode(&bytes[3..]);
        assert_eq!(format!("{a}{b}"), "Grüße");
        assert_eq!(a, "Gr");
    }
    /// Ende-zu-Ende: echte zsh + Kompass-Integration + Wächter, nur ohne Fenster.
    #[test]
    fn echte_zsh_mit_waechter() {
        use std::time::{Duration, Instant};
        let play = std::env::temp_dir().join(format!("kompass-e2e-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&play);
        std::fs::create_dir_all(&play).unwrap();
        std::fs::write(play.join("a.txt"), "A").unwrap();
        std::fs::write(play.join("wichtig.txt"), "bleibt").unwrap();

        let zdotdir = install_shell_files().unwrap();
        let pair = native_pty_system().openpty(PtySize { rows: 30, cols: 120, pixel_width: 0, pixel_height: 0 }).unwrap();
        let mut cmd = shell_command(&zdotdir);
        cmd.env("KOMPASS_BIN", "/bin/echo"); // statt echtem Papierkorb nur anzeigen
        cmd.cwd(&play);
        // Eigenes „Zuhause“ für die Test-Shell: weder deine .zshrc noch dein Verlauf werden berührt.
        let fake_home = play.join("home");
        std::fs::create_dir_all(&fake_home).unwrap();
        cmd.env("KOMPASS_USER_ZDOTDIR", &fake_home);
        let mut child = pair.slave.spawn_command(cmd).unwrap();
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().unwrap();
        let mut writer = pair.master.take_writer().unwrap();

        // Lese-Thread schickt alles in einen Kanal (mpsc = „viele Sender, ein Empfänger“).
        let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 || tx.send(buf[..n].to_vec()).is_err() { break; }
            }
        });

        let mut scanner = OscScanner::default();
        let mut done: Vec<String> = Vec::new(); // „Befehl fertig“-Meldungen
        // Tippt eine Zeile und beantwortet die Wächter-Anfrage mit `decide`.
        // Gibt (Urteil, Ausgabe, Live-Meldungen beim Tippen) zurück.
        let mut run = |line: &str, decide: &dyn Fn(&Verdict) -> String| -> (Verdict, String, Vec<String>) {
            // Zeichen einzeln tippen wie ein Mensch, dann Enter.
            for ch in line.chars() {
                writer.write_all(ch.to_string().as_bytes()).unwrap();
                std::thread::sleep(Duration::from_millis(15));
            }
            writer.write_all(b"\r").unwrap();
            let mut text = Vec::new();
            let mut verdict = None;
            let mut live: Vec<String> = Vec::new();
            let until = Instant::now() + Duration::from_secs(4);
            while Instant::now() < until {
                if let Ok(chunk) = rx.recv_timeout(Duration::from_millis(300)) {
                    let (vis, reqs) = scanner.feed(&chunk);
                    text.extend(vis);
                    for (kind, r) in reqs {
                        let s = String::from_utf8_lossy(&r).to_string();
                        if kind == 2 {
                            done.push(s);
                            continue;
                        }
                        if kind == 1 {
                            live.push(s.splitn(3, '\x1f').nth(2).unwrap_or_default().to_string());
                            continue;
                        }
                        if kind != 0 {
                            continue;
                        }
                        let mut p = s.splitn(3, ';');
                        let c = String::from_utf8(B64.decode(p.next().unwrap()).unwrap()).unwrap();
                        let d = String::from_utf8(B64.decode(p.next().unwrap()).unwrap()).unwrap();
                        let known = p.next() != Some("0");
                        // Klarsprache wie im Router: übersetzen und in die Zeile setzen.
                        if let Some(sugg) = nl::detect(&c, known).and_then(|q| nl::translate(&q, Path::new(&d))) {
                            writer.write_all(format!("e{}.", B64.encode(&sugg.command)).as_bytes()).unwrap();
                            verdict = Some(Verdict { level: Level::Safe, findings: vec![], trash: None });
                            continue;
                        }
                        let v = guard::check(&c, Path::new(&d));
                        writer.write_all(decide(&v).as_bytes()).unwrap();
                        verdict = Some(v);
                    }
                } else if verdict.is_some() {
                    break; // Antwort gegeben und Shell ist ruhig
                }
            }
            (verdict.expect("keine Wächter-Anfrage erhalten"), String::from_utf8_lossy(&text).to_string(), live)
        };

        std::thread::sleep(Duration::from_millis(1500)); // Shell hochfahren lassen
        let (v, out, live) = run("ls", &|_| "r.".into());
        assert_eq!(v.level, Level::Safe);
        assert!(out.contains("wichtig.txt"), "ls-Ausgabe fehlt: {out}");

        let (v, out, _) = run("rm -f a.txt", &|v| format!("r{}.", B64.encode(&v.trash.as_ref().unwrap().rewritten)));
        assert_eq!(v.level, Level::Danger);
        assert!(out.contains("--kompass-trash -- a.txt"), "Papierkorb-Umleitung fehlt: {out}");
        assert!(play.join("a.txt").exists(), "rm darf nicht wirklich löschen");

        let (v, out, _) = run("echo weg > wichtig.txt", &|_| "c.".into());
        assert_eq!(v.level, Level::Danger);
        assert!(out.contains("Abgebrochen"), "Abbruch-Meldung fehlt: {out}");
        assert_eq!(std::fs::read_to_string(play.join("wichtig.txt")).unwrap(), "bleibt");

        // Klarsprache: „zeige alle dateien“ wird nicht ausgeführt, sondern in „ls -a“ übersetzt …
        let (_, out, nl_live) = run("zeige alle dateien", &|_| "r.".into());
        assert!(!out.contains("command not found"), "Klarsprache wurde ausgeführt: {out}");
        assert!(nl_live.iter().any(|l| l == "ls -a"), "Übersetzung nicht in der Zeile: {nl_live:?}");
        // … und das zweite Enter führt den Befehl aus.
        let (v, out, _) = run("", &|_| "r.".into());
        assert_eq!(v.level, Level::Safe);
        assert!(out.contains("wichtig.txt"), "ls -a lief nicht: {out}");

        // Beim Tippen kamen Live-Meldungen mit der wachsenden Zeile.
        assert!(live.iter().any(|l| l == "l") && live.iter().any(|l| l == "ls"), "Live-Meldungen: {live:?}");

        // Nach jedem ausgeführten Befehl meldet die Shell „fertig“ mit Rückgabewert 0.
        assert!(done.len() >= 2 && done.iter().all(|d| d.starts_with("0\x1f")), "Fertig-Meldungen: {done:?}");

        // Verlauf landet im Zuhause, nicht im Kompass-Ordner.
        writer.write_all(b"exit\r").unwrap();
        std::thread::sleep(Duration::from_millis(300));
        let _ = writer.write_all(b"r.");
        std::thread::sleep(Duration::from_millis(500));
        let hist = std::fs::read_to_string(fake_home.join(".zsh_history")).unwrap_or_default();
        assert!(hist.contains("echo weg"), "Verlauf nicht im Zuhause gelandet");
        let _ = child.kill();
    }
}
