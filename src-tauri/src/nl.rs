//! Klarsprache → Befehl (lokal, ohne Internet).
//!
//! Beispiel: „zeige alle pdf dateien“ → `find . -iname "*.pdf"`.
//! Funktioniert mit einfachen Regeln: Schlüsselwörter erkennen, Namen
//! herausziehen (gibt es die Datei hier wirklich?), Befehl zusammensetzen.
//! Was die Regeln nicht verstehen, kann optional die KI übernehmen (ki.rs).

use crate::i18n::is_en;
use serde::Serialize;
use std::path::Path;

/// Ein Übersetzungsvorschlag.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    pub command: String,
    /// Was der Befehl tut, in Klartext.
    pub text: String,
    /// Fehlt noch etwas (z. B. ein Dateiname)? Dann steht der Befehl nur halb da.
    pub needs_input: bool,
    /// "local" oder "ki"
    pub source: &'static str,
}

/// Ist das Klarsprache? `known` = die zsh kennt das erste Wort als Befehl.
/// Gibt den eigentlichen Wunsch zurück (ohne führendes „?“).
pub fn detect(line: &str, known: bool) -> Option<String> {
    let t = line.trim();
    // `?` am Anfang erzwingt Klarsprache, z. B. „? wie viel platz ist frei“
    if let Some(rest) = t.strip_prefix('?') {
        let rest = rest.trim();
        return (!rest.is_empty()).then(|| rest.to_string());
    }
    if known || t.is_empty() {
        return None;
    }
    let first = t.split_whitespace().next()?;
    // Klarsprache beginnt mit einem Wort aus Buchstaben (kein Pfad, keine Variable …)
    if !first.chars().all(|c| c.is_alphabetic() || c == '-') {
        return None;
    }
    Some(t.trim_end_matches(['?', '!', '.']).to_string())
}

/// Sieht eine Zeile wie ein Satz aus, obwohl sie mit einem echten Befehl beginnt?
/// Beispiel: „find all pdf files“ (`find` gibt es wirklich) oder „open the downloads folder“.
/// Nur dann, wenn keine Shell-Zeichen vorkommen und typische Satzwörter dabei sind.
pub fn looks_like_sentence(line: &str) -> bool {
    let t = line.trim();
    if t.chars().any(|c| "-./~*|&;<>$=\"'`(){}[]".contains(c)) {
        return false;
    }
    let words: Vec<String> = t.split_whitespace().map(fold).collect();
    const MARKERS: &[&str] = &[
        "the", "a", "an", "all", "my", "me", "please", "files", "folder", "folders", "everything",
        "alle", "alles", "die", "der", "das", "den", "mir", "meine", "meinen", "bitte", "dateien", "ordner",
    ];
    words.len() >= 3 && words.iter().skip(1).any(|w| MARKERS.contains(&w.as_str()))
}

/// Vereinfachte Schreibweise zum Vergleichen: klein, ä→ae, ß→ss.
fn fold(s: &str) -> String {
    s.to_lowercase()
        .replace('ä', "ae")
        .replace('ö', "oe")
        .replace('ü', "ue")
        .replace('ß', "ss")
}

/// Die zerlegte Anfrage.
struct Q<'a> {
    /// vereinfachte Wörter (zum Erkennen)
    words: Vec<String>,
    /// Original-Wörter, Satzzeichen am Rand entfernt (für Namen)
    orig: Vec<String>,
    /// Text in Anführungszeichen, z. B. „Hallo Welt“
    quoted: Vec<String>,
    /// alles vereinfacht in einer Zeile
    flat: String,
    cwd: &'a Path,
}

impl<'a> Q<'a> {
    fn new(text: &str, cwd: &'a Path) -> Self {
        let mut quoted = Vec::new();
        let mut rest = String::new();
        let mut cur: Option<String> = None;
        for c in text.chars() {
            match (&mut cur, c) {
                (None, '"' | '„' | '“' | '\'' | '»' | '«' | '‚' | '‘') => cur = Some(String::new()),
                (Some(_), '"' | '“' | '”' | '\'' | '«' | '»' | '‘' | '’') => {
                    quoted.push(cur.take().unwrap());
                    rest.push(' ');
                }
                (Some(s), c) => s.push(c),
                (None, c) => rest.push(c),
            }
        }
        if let Some(s) = cur {
            quoted.push(s); // nicht geschlossenes Anführungszeichen
        }
        let orig: Vec<String> = rest
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| matches!(c, ',' | ';' | '!' | '?' | ':' | '(' | ')')).trim_end_matches('.').to_string())
            .filter(|w| !w.is_empty())
            .collect();
        let words: Vec<String> = orig.iter().map(|w| fold(w)).collect();
        let flat = words.join(" ");
        Q { words, orig, quoted, flat, cwd }
    }

    /// Beginnt eines der Wörter mit einem dieser Wortstämme?
    fn has(&self, stems: &[&str]) -> bool {
        self.words.iter().any(|w| stems.iter().any(|s| w.starts_with(s)))
    }

    /// Kommt eines dieser Wörter genau so vor (nicht nur als Wortanfang)?
    fn word(&self, list: &[&str]) -> bool {
        self.words.iter().any(|w| list.contains(&w.as_str()))
    }

    /// Kommt eine dieser Wortfolgen vor?
    fn phrase(&self, phrases: &[&str]) -> bool {
        let padded = format!(" {} ", self.flat);
        phrases.iter().any(|p| padded.contains(&format!(" {p} ")) || padded.contains(&format!(" {p}")))
    }

    /// Gibt es diesen Namen hier im Ordner? Liefert die echte Schreibweise.
    fn existing(&self, name: &str) -> Option<String> {
        // Erst im Ordner nach der echten Schreibweise suchen: macOS unterscheidet
        // Groß/Klein beim Zugriff nicht, wir wollen aber „Fotos“ statt „fotos“ anzeigen.
        let wanted = name.to_lowercase();
        let entries: Vec<String> = std::fs::read_dir(self.cwd)
            .map(|d| d.take(3000).flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect())
            .unwrap_or_default();
        if let Some(exact) = entries.iter().find(|n| *n == name) {
            return Some(exact.clone());
        }
        if let Some(similar) = entries.iter().find(|n| n.to_lowercase() == wanted) {
            return Some(similar.clone());
        }
        // Pfade wie `Fotos/2024` oder `~/x`
        (name.contains('/') && crate::guard::resolve(name, self.cwd).exists()).then(|| name.to_string())
    }

    /// Mögliche Datei-/Ordnernamen in der Reihenfolge, in der sie vorkommen.
    fn names(&self) -> Vec<String> {
        let mut out: Vec<String> = self.quoted.iter().filter(|q| !q.trim().is_empty()).cloned().collect();
        let markers = ["ordner", "datei", "verzeichnis", "namens", "heisst", "projekt", "bild", "foto", "dokument", "app", "programm", "folder", "file", "directory", "named", "called", "project", "image", "photo", "document", "program"];
        for (i, w) in self.orig.iter().enumerate() {
            let folded = &self.words[i];
            let after_marker = i > 0 && markers.iter().any(|m| self.words[i - 1].starts_with(m)) && !STOP.contains(&folded.as_str()) && !is_keyword(folded);
            let looks_like_path = w.contains('.') && w.len() > 1 || w.contains('/') || w.starts_with('~');
            if let Some(real) = self.existing(w) {
                if !STOP.contains(&folded.as_str()) {
                    out.push(real);
                    continue;
                }
            }
            if looks_like_path || after_marker {
                out.push(w.clone());
            }
        }
        out.dedup();
        out
    }

    /// Das erste brauchbare Wort nach einem dieser Wörter (z. B. Ziel nach „nach“).
    fn after(&self, markers: &[&str]) -> Option<String> {
        let mut found = None;
        for (i, w) in self.words.iter().enumerate() {
            if markers.contains(&w.as_str()) {
                found = self.orig[i + 1..]
                    .iter()
                    .zip(&self.words[i + 1..])
                    .find(|(_, f)| !STOP.contains(&f.as_str()) && !matches!(f.as_str(), "ordner" | "datei" | "verzeichnis" | "den" | "die" | "das"))
                    .map(|(o, _)| o.clone());
            }
        }
        // Echte Schreibweise verwenden, falls es den Namen hier gibt.
        found.map(|o| self.existing(&o).unwrap_or(o))
    }

    fn number(&self) -> Option<u32> {
        self.words.iter().find_map(|w| w.parse().ok())
    }
}

/// Füllwörter, die nie ein Name sind.
const STOP: &[&str] = &[
    "der", "die", "das", "den", "dem", "des", "ein", "eine", "einen", "einem", "einer", "mir", "mich", "mal", "bitte", "alle",
    "alles", "allen", "im", "in", "ins", "nach", "von", "vom", "aus", "zu", "zum", "zur", "mit", "und", "oder", "hier", "da",
    "diese", "dieser", "diesen", "dieses", "jetzt", "doch", "ganz", "neu", "neuen", "neue", "neuer", "neues", "leere", "leeren",
    "leer", "um", "an", "auf", "fuer", "was", "wie", "wo", "welche", "welcher", "ist", "sind", "gibt", "es", "ich", "meine",
    "meinen", "mein", "meiner", "als", "aktuellen", "aktuelle", "aktuell", "ordner", "datei", "dateien", "inhalt", "gross",
    "namens", "heisst", "name", "namen", "gerade",
    // englisch
    "the", "a", "an", "all", "my", "me", "please", "to", "into", "in", "of", "from", "with", "and", "or", "here", "this", "that",
    "these", "those", "now", "new", "empty", "named", "called", "folder", "folders", "file", "files", "directory", "content",
    "contents", "what", "which", "where", "is", "are", "there", "it", "i", "current", "everything", "size", "big", "for", "on", "at", "up",
];

/// Typische Befehls-Verben – nie ein Dateiname.
fn is_keyword(w: &str) -> bool {
    const K: &[&str] = &["zeig", "list", "loesch", "entfern", "kopier", "verschieb", "benenn", "erstell", "anleg", "leg", "mach", "oeffne", "geh", "wechsl", "such", "find", "pack", "entpack", "zipp",
        "show", "delete", "remove", "copy", "move", "rename", "create", "make", "open", "go", "change", "search", "compress", "extract", "unzip"];
    K.iter().any(|k| w.starts_with(k))
}

/// Shell-sicher in Anführungszeichen setzen, wenn nötig.
fn sh(s: &str) -> String {
    if !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || "._-/~+@%".contains(c)) {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', r"'\''"))
    }
}

fn ok(command: impl Into<String>, text: impl Into<String>) -> Option<Suggestion> {
    Some(Suggestion { command: command.into(), text: text.into(), needs_input: false, source: "local" })
}

fn partial(command: impl Into<String>, text: impl Into<String>) -> Option<Suggestion> {
    Some(Suggestion { command: command.into(), text: text.into(), needs_input: true, source: "local" })
}

/// Bekannte Orte: „schreibtisch“ → ~/Desktop
fn place(q: &Q) -> Option<(&'static str, &'static str)> {
    const PLACES: &[(&str, &str, &str)] = &[
        ("schreibtisch", "~/Desktop", "Schreibtisch"),
        ("documents", "~/Documents", "Documents"),
        ("pictures", "~/Pictures", "Pictures"),
        ("music", "~/Music", "Music"),
        ("movies", "~/Movies", "Movies"),
        ("applications", "/Applications", "Applications"),
        ("playground", "~/kompass-spielplatz", "Playground"),
        ("desktop", "~/Desktop", "Schreibtisch"),
        ("download", "~/Downloads", "Downloads"),
        ("dokument", "~/Documents", "Dokumente"),
        ("bilder", "~/Pictures", "Bilder"),
        ("musik", "~/Music", "Musik"),
        ("filme", "~/Movies", "Filme"),
        ("programme", "/Applications", "Programme"),
        ("spielplatz", "~/kompass-spielplatz", "Spielplatz"),
    ];
    PLACES.iter().find(|(k, _, _)| q.has(&[k])).map(|(_, p, n)| (*p, *n))
}

/// Dateiarten für „finde alle pdfs“.
fn file_types(q: &Q) -> Option<(Vec<&'static str>, &'static str)> {
    const T: &[(&[&str], &[&str], &str)] = &[
        (&["pdf"], &["pdf"], "PDF-Dateien"),
        (&["foto", "bild", "jpg", "jpeg", "photo", "image", "picture"], &["jpg", "jpeg", "png", "heic"], "Bilder"),
        (&["png"], &["png"], "PNG-Bilder"),
        (&["video", "film", "mp4", "mov", "movie"], &["mp4", "mov", "m4v"], "Videos"),
        (&["musik", "mp3", "lied", "song", "music"], &["mp3", "m4a", "wav"], "Musikdateien"),
        (&["word", "docx"], &["doc", "docx"], "Word-Dateien"),
        (&["excel", "xlsx", "tabelle", "spreadsheet"], &["xls", "xlsx", "csv"], "Tabellen"),
        (&["text", "txt", "notiz", "note"], &["txt", "md"], "Textdateien"),
        (&["zip", "archiv", "archive"], &["zip", "tar", "gz"], "Archive"),
        (&["indesign", "indd"], &["indd", "idml"], "InDesign-Dateien"),
        (&["photoshop", "psd"], &["psd", "psb"], "Photoshop-Dateien"),
        (&["illustrator"], &["ai"], "Illustrator-Dateien"),
        (&["tif"], &["tif", "tiff"], "TIFF-Bilder"),
        (&["eps"], &["eps"], "EPS-Dateien"),
    ];
    T.iter().find(|(keys, _, _)| q.has(keys)).map(|(_, ext, name)| (ext.to_vec(), *name))
}

fn find_by_ext(ext: &[&str]) -> String {
    if ext.len() == 1 {
        return format!("find . -type f -iname '*.{}'", ext[0]);
    }
    let parts: Vec<String> = ext.iter().map(|e| format!("-iname '*.{e}'")).collect();
    format!("find . -type f \\( {} \\)", parts.join(" -o "))
}

/// Hauptfunktion: versucht, den Wunsch in einen Befehl zu übersetzen.
/// Auf Englisch kommt der Erklärtext aus dem englischen Befehlslexikon (explain.rs).
pub fn translate(text: &str, cwd: &Path) -> Option<Suggestion> {
    let mut s = translate_rules(text, cwd)?;
    if is_en() {
        s.text = if s.needs_input {
            format!("Fill in what is missing after “{}”", s.command.trim())
        } else {
            crate::explain::explain(&s.command, cwd).summary.trim_end_matches('.').to_string()
        };
    }
    Some(s)
}

fn translate_rules(text: &str, cwd: &Path) -> Option<Suggestion> {
    let q = Q::new(text, cwd);
    if q.words.is_empty() {
        return None;
    }
    let names = q.names();
    let first = names.first().cloned();
    let is_dir = |n: &str| q.cwd.join(n).is_dir();

    // ── Orientierung ──
    if q.phrase(&["wo bin ich", "welcher ordner", "aktueller ordner", "aktuellen ordner", "wo befinde ich mich", "in welchem ordner", "where am i", "which folder", "current folder", "current directory"]) {
        return ok("pwd", "Zeigt, in welchem Ordner du gerade bist");
    }

    // ── Hilfe zu einem Befehl ──
    if q.has(&["anleitung", "handbuch", "hilfe", "manual", "help"]) && q.words.len() >= 2 {
        if let Some(cmd) = q.orig.iter().zip(&q.words).find(|(_, w)| crate::explain::known_names().any(|k| k == w.as_str())).map(|(o, _)| o.clone()) {
            return ok(format!("man {cmd}"), format!("Öffnet die Anleitung zu „{cmd}“ (q beendet)"));
        }
    }

    // ── Systeminfos ──
    if q.has(&["speicher", "festplatte", "platz", "space", "disk", "storage"]) && q.has(&["frei", "voll", "uebrig", "wieviel", "viel", "platz", "speicherplatz", "free", "left", "much", "full", "available"]) && !q.has(&["ordner", "datei", "folder", "file"]) {
        return ok("df -h /", "Zeigt, wie viel Platz auf deiner Festplatte frei ist");
    }
    if q.has(&["groesste", "groessten", "riesig", "largest", "biggest"]) && q.has(&["datei", "ordner", "platzfresser", "file", "folder"]) || q.has(&["platzfresser"]) {
        return ok("du -ah . 2>/dev/null | sort -rh | head -n 10", "Zeigt die 10 größten Dateien und Ordner hier");
    }
    if q.has(&["gross", "groesse", "belegt", "verbraucht", "size"]) && !q.has(&["groesste", "largest", "biggest"]) && (first.is_some() || q.has(&["ordner", "folder"])) || q.phrase(&["how big"]) && first.is_some() {
        let target = first.clone().unwrap_or_else(|| ".".into());
        return ok(format!("du -sh {}", sh(&target)), format!("Zeigt, wie viel Platz „{target}“ belegt"));
    }
    if q.has(&["akku", "batterie", "battery"]) {
        return ok("pmset -g batt", "Zeigt den Ladestand deines Akkus");
    }
    if q.has(&["macos", "version", "systemversion"]) && q.has(&["mac", "macos", "system", "version"]) && !q.has(&["git", "node", "python"]) {
        return ok("sw_vers", "Zeigt, welche macOS-Version du hast");
    }
    if q.phrase(&["wie lange", "seit wann", "how long"]) && (q.has(&["laeuft", "an", "eingeschaltet", "running"]) || q.word(&["on"])) || q.has(&["uptime"]) {
        return ok("uptime", "Zeigt, wie lange dein Mac schon läuft");
    }
    if q.has(&["programme", "prozesse", "prozess", "rechenleistung", "cpu", "auslastung", "programs", "apps", "processes"]) && q.has(&["laufen", "laeuft", "aktiv", "zeig", "welche", "auslastung", "cpu", "prozesse", "running", "show", "which", "processes"]) && !q.has(&["beend", "stopp", "kill", "oeffn", "start", "quit", "close", "open"]) {
        return ok("top -o cpu -l 1 -n 10 -stats pid,command,cpu,mem", "Zeigt die 10 Programme, die gerade am meisten arbeiten");
    }
    if q.has(&["beend", "stopp", "schliess", "abschiess", "kill", "quit", "close", "stop", "terminate"]) && q.has(&["programm", "app", "prozess", "program", "process"]) || q.has(&["killall"]) {
        let app = q.after(&["programm", "app", "prozess", "program", "process"]).or(first.clone());
        return match app {
            Some(a) => ok(format!("killall {}", sh(&a)), format!("Beendet das Programm „{a}“ sofort (ungespeicherte Arbeit geht verloren)")),
            None => partial("killall ", "Beendet ein Programm – tippe noch seinen Namen dazu"),
        };
    }
    if q.has(&["ip"]) || q.phrase(&["ip adresse", "ip-adresse", "ip address"]) {
        if q.has(&["oeffentlich", "extern", "internet", "public", "external"]) {
            return ok("curl -s https://ifconfig.me; echo", "Zeigt deine öffentliche IP-Adresse (fragt dazu einen Dienst im Internet)");
        }
        return ok("ipconfig getifaddr en0", "Zeigt die IP-Adresse deines Macs im Heimnetz");
    }
    if q.has(&["internet", "online", "verbindung", "netz", "connection"]) && (q.has(&["pruef", "teste", "test", "geht", "funktioniert", "habe", "bin", "check", "working"]) || q.word(&["am", "do", "is"])) {
        return ok("ping -c 3 apple.com", "Prüft mit 3 kleinen Paketen, ob das Internet erreichbar ist");
    }
    if (q.has(&["uhrzeit", "spaet", "datum", "heute", "today"]) || q.word(&["tag", "time", "date"])) && !q.has(&["kalender", "calendar", "uptime"]) && q.words.len() <= 5 {
        return ok("date", "Zeigt Datum und Uhrzeit");
    }
    if q.has(&["kalender", "monat", "calendar", "month"]) {
        return ok("cal", "Zeigt den Kalender dieses Monats");
    }
    if q.has(&["wach", "awake"]) && q.has(&["mac", "halt", "bleib", "keep", "stay"]) {
        return ok("caffeinate -d", "Hält den Mac wach, bis du Ctrl+C drückst");
    }
    if q.has(&["bildschirmfoto", "screenshot"]) {
        return ok("screencapture -i ~/Desktop/bildschirmfoto.png", "Macht ein Bildschirmfoto eines Bereichs (aufziehen mit der Maus) und legt es auf den Schreibtisch");
    }
    if q.has(&["verlauf", "history"]) || q.phrase(&["letzte befehle", "letzten befehle", "was habe ich getippt", "last commands", "recent commands", "what did i type"]) {
        return ok("history", "Zeigt die Befehle, die du zuletzt getippt hast");
    }
    if q.has(&["aufraeum", "leer", "clear", "clean"]) && q.has(&["fenster", "bildschirm", "terminal", "window", "screen"]) {
        return ok("clear", "Räumt das Fenster auf (der Verlauf bleibt)");
    }
    if q.has(&["sag", "sprich", "vorles", "say", "speak"]) {
        let text = q.quoted.first().cloned().unwrap_or_else(|| {
            let skip = q.words.iter().position(|w| ["sag", "sprich", "vorles", "say", "speak"].iter().any(|k| w.starts_with(k))).unwrap_or(0);
            q.orig[skip + 1..].join(" ")
        });
        if !text.trim().is_empty() {
            return ok(format!("say {}", sh(&text)), "Dein Mac spricht den Text laut aus");
        }
    }
    if q.has(&["versteckt", "hidden"]) && q.has(&["finder"]) {
        let show = !q.has(&["ausblend", "verstecken", "nicht", "hide", "not", "don"]);
        return ok(
            format!("defaults write com.apple.finder AppleShowAllFiles -bool {} && killall Finder", if show { "true" } else { "false" }),
            if show { "Lässt den Finder versteckte Dateien anzeigen (Finder startet dafür kurz neu)" } else { "Versteckt die versteckten Dateien im Finder wieder" },
        );
    }
    if q.has(&["git", "geaendert", "aenderungen", "changed", "changes"]) && q.has(&["status", "geaendert", "aenderungen", "was", "changed", "changes", "what"]) {
        return ok("git status", "Zeigt, was sich in diesem git-Projekt geändert hat");
    }

    // ── Navigieren ──
    let go = q.has(&["geh", "wechsl", "wechsel", "spring", "navigier", "zurueck", "rein", "hinein", "change", "switch", "navigate", "jump", "enter", "back"]) || q.word(&["go"]) || q.words.first().map(|w| w == "zu" || w == "nach" || w == "in" || w == "to").unwrap_or(false);
    if go || q.phrase(&["nach hause", "nach oben", "eine ebene", "go home", "go up", "one level"]) {
        if q.phrase(&["nach hause", "home", "benutzerordner", "heimordner", "home folder"]) {
            return ok("cd", "Bringt dich zurück in deinen Benutzerordner");
        }
        if q.phrase(&["nach oben", "eine ebene", "hoch", "darueber", "uebergeordnet", "up", "one level", "parent"]) {
            return ok("cd ..", "Geht eine Ordner-Ebene nach oben");
        }
        if q.has(&["zurueck", "vorherig", "vorher", "back", "previous"]) && first.is_none() {
            return ok("cd -", "Springt zurück in den Ordner, in dem du vorher warst");
        }
        if let Some((p, n)) = place(&q) {
            return ok(format!("cd {p}"), format!("Wechselt in deinen Ordner „{n}“"));
        }
        if let Some(n) = &first {
            if is_dir(n) || n.contains('/') || n.starts_with('~') {
                return ok(format!("cd {}", sh(n)), format!("Wechselt in den Ordner „{n}“"));
            }
        }
        if q.has(&["ordner", "verzeichnis", "folder", "directory"]) {
            return partial("cd ", "Wechselt in einen Ordner – tippe noch den Namen dazu (Tab ergänzt!)");
        }
    }

    // ── Öffnen ──
    if q.has(&["oeffne", "oeffnen", "starte", "starten", "open", "launch", "start"]) {
        if let Some((p, n)) = place(&q) {
            return ok(format!("open {p}"), format!("Öffnet „{n}“ im Finder"));
        }
        if q.has(&["finder"]) || (q.has(&["ordner", "folder"]) && first.is_none()) {
            return ok("open .", "Öffnet den aktuellen Ordner im Finder");
        }
        if let Some(n) = &first {
            if q.existing(n).is_some() {
                return ok(format!("open {}", sh(n)), format!("Öffnet „{n}“ mit dem passenden Programm"));
            }
        }
        if let Some(app) = q.after(&["app", "programm", "program"]).or_else(|| q.orig.last().cloned().filter(|w| w.chars().next().map(char::is_uppercase).unwrap_or(false))) {
            return ok(format!("open -a {}", sh(&app)), format!("Startet das Programm „{app}“"));
        }
    }

    // ── Anlegen ──
    if q.has(&["erstell", "anleg", "leg", "mach", "neu", "neuen", "neue", "erzeug", "create", "make", "new", "add"]) {
        if q.has(&["ordner", "verzeichnis", "folder", "directory"]) {
            return match &first {
                Some(n) => ok(format!("mkdir -p {}", sh(n)), format!("Legt den Ordner „{n}“ an")),
                None => partial("mkdir ", "Legt einen Ordner an – tippe noch den Namen dazu"),
            };
        }
        if q.has(&["datei", "textdatei", "notiz", "file", "note"]) {
            return match &first {
                Some(n) => ok(format!("touch {}", sh(n)), format!("Legt die leere Datei „{n}“ an")),
                None => partial("touch ", "Legt eine leere Datei an – tippe noch den Namen dazu"),
            };
        }
    }

    // ── Schreiben ──
    if q.has(&["schreib", "write", "append"]) && !q.quoted.is_empty() {
        let text = &q.quoted[0];
        let target = q.after(&["in", "nach", "to", "into"]).or_else(|| names.iter().find(|n| *n != text).cloned());
        if let Some(t) = target {
            return ok(format!("echo {} >> {}", sh(text), sh(&t)), format!("Hängt den Text hinten an „{t}“ an"));
        }
    }

    // ── Löschen ──
    if q.has(&["loesch", "entfern", "wegwerf", "papierkorb", "weg", "delete", "remove", "trash", "erase"]) && !q.has(&["papierkorb leer"]) {
        if names.is_empty() {
            return partial("rm ", "Löscht Dateien – tippe noch den Namen dazu (Kompass legt sie in den Papierkorb)");
        }
        let any_dir = names.iter().any(|n| is_dir(n));
        let list: Vec<String> = names.iter().map(|n| sh(n)).collect();
        return ok(
            format!("rm {}{}", if any_dir { "-r " } else { "" }, list.join(" ")),
            format!("Löscht {} – Kompass legt alles in den Papierkorb", names.iter().map(|n| format!("„{n}“")).collect::<Vec<_>>().join(", ")),
        );
    }

    // ── Kopieren / Verschieben / Umbenennen ──
    if q.has(&["benenn", "umbenenn", "rename"]) {
        let to = q.after(&["in", "zu", "nach", "als", "to", "into", "as"]);
        let from = names.iter().find(|n| Some(*n) != to.as_ref()).cloned();
        return match (from, to) {
            (Some(f), Some(t)) => ok(format!("mv {} {}", sh(&f), sh(&t)), format!("Benennt „{f}“ in „{t}“ um")),
            _ => partial("mv ", "Benennt um: erst der alte Name, dann der neue"),
        };
    }
    let copy = q.has(&["kopier", "dupliz", "copy", "duplicate"]);
    if copy || q.has(&["verschieb", "beweg"]) || q.word(&["move"]) {
        let to = q.after(&["nach", "in", "zu", "ins", "to", "into"]).map(|t| place(&Q::new(&t, cwd)).map(|(p, _)| p.to_string()).unwrap_or(t));
        let sources: Vec<String> = names.iter().filter(|n| Some(*n) != to.as_ref()).cloned().collect();
        let (cmd, verb) = if copy { ("cp", "Kopiert") } else { ("mv", "Verschiebt") };
        return match (sources.is_empty(), to) {
            (false, Some(t)) => {
                let r = if copy && sources.iter().any(|s| is_dir(s)) { " -R" } else { "" };
                let list: Vec<String> = sources.iter().map(|s| sh(s)).collect();
                ok(format!("{cmd}{r} {} {}", list.join(" "), sh(&t)), format!("{verb} {} nach „{t}“", sources.iter().map(|n| format!("„{n}“")).collect::<Vec<_>>().join(", ")))
            }
            _ => partial(format!("{cmd} "), format!("{verb} – erst was, dann wohin")),
        };
    }

    // ── Suchen ──
    if q.has(&["such", "find", "search", "look", "locate"]) || q.phrase(&["wo ist", "wo liegt", "wo sind", "where is", "where are"]) {
        // Text in Dateien suchen: „suche nach "Rechnung" in allen dateien“
        if let Some(word) = q.quoted.first() {
            let target = q.after(&["in"]).filter(|t| q.existing(t).is_some());
            return match target {
                Some(t) if !is_dir(&t) => ok(format!("grep -n {} {}", sh(word), sh(&t)), format!("Sucht „{word}“ in „{t}“ und zeigt die Zeilen")),
                Some(t) => ok(format!("grep -rni {} {}", sh(word), sh(&t)), format!("Sucht „{word}“ in allen Dateien in „{t}“")),
                None => ok(format!("grep -rni {} .", sh(word)), format!("Sucht „{word}“ in allen Dateien hier und darunter (Groß/Klein egal)")),
            };
        }
        if let Some((ext, name)) = file_types(&q) {
            let mut cmd = find_by_ext(&ext);
            let mut text = format!("Findet alle {name} hier und in allen Unterordnern");
            if let Some((p, n)) = place(&q) {
                cmd = cmd.replacen("find .", &format!("find {p}"), 1);
                text = format!("Findet alle {name} in „{n}“");
            }
            return ok(cmd, text);
        }
        let name = q.after(&["namens", "datei", "heisst", "nach", "named", "called", "for", "file"]).or(first.clone());
        if let Some(n) = name {
            return ok(format!("find . -iname '*{n}*'"), format!("Sucht Dateien und Ordner, deren Name „{n}“ enthält"));
        }
        return partial("find . -iname '**'", "Sucht nach Namen – schreib den Suchbegriff zwischen die Sternchen");
    }

    // ── Packen ──
    if q.has(&["entpack", "auspack", "unzip", "extract", "unpack", "decompress"]) {
        return match &first {
            Some(n) if n.ends_with(".zip") => ok(format!("unzip {}", sh(n)), format!("Entpackt „{n}“ hierher")),
            Some(n) => ok(format!("tar -xzf {}", sh(n)), format!("Entpackt das Archiv „{n}“ hierher")),
            None => partial("unzip ", "Entpackt ein ZIP – tippe noch den Namen dazu"),
        };
    }
    if q.has(&["zip", "pack", "komprimier", "archiv", "compress", "archive"]) {
        return match &first {
            Some(n) => {
                let base = n.trim_end_matches('/');
                ok(format!("zip -r {}.zip {}", sh(base), sh(n)), format!("Packt „{n}“ in die ZIP-Datei „{base}.zip“"))
            }
            None => partial("zip -r archiv.zip ", "Packt Dateien in ein ZIP – tippe noch dazu, was rein soll"),
        };
    }

    // ── Herunterladen ──
    if q.has(&["herunterlad", "runterlad", "download", "lade", "fetch"]) {
        if let Some(url) = q.orig.iter().find(|w| w.starts_with("http")) {
            return ok(format!("curl -LO {url}"), "Lädt die Datei herunter und speichert sie hier");
        }
    }

    // ── Zählen ──
    if q.has(&["zaehl", "anzahl", "wieviele", "count"]) || q.phrase(&["wie viele", "how many"]) {
        if q.has(&["zeile", "line"]) {
            if let Some(n) = &first {
                return ok(format!("wc -l {}", sh(n)), format!("Zählt die Zeilen in „{n}“"));
            }
        }
        return ok("ls | wc -l", "Zählt, wie viele Dinge in diesem Ordner liegen");
    }

    // ── Inhalt anzeigen ──
    if let Some(n) = first.clone().filter(|n| q.cwd.join(n).is_file()) {
        if q.has(&["erste", "anfang", "oben", "first", "beginning", "top"]) {
            let k = q.number().unwrap_or(10);
            return ok(format!("head -n {k} {}", sh(&n)), format!("Zeigt die ersten {k} Zeilen von „{n}“"));
        }
        if q.has(&["letzte", "ende", "unten", "last", "end", "bottom"]) {
            let k = q.number().unwrap_or(10);
            return ok(format!("tail -n {k} {}", sh(&n)), format!("Zeigt die letzten {k} Zeilen von „{n}“"));
        }
        if q.has(&["bearbeit", "aender", "editier", "edit"]) {
            return ok(format!("nano {}", sh(&n)), format!("Öffnet „{n}“ im einfachen Editor (Ctrl+X beendet)"));
        }
        if q.has(&["zeig", "lies", "lese", "inhalt", "was steht", "anzeig", "ausgeb", "show", "read", "display", "print", "content", "view"]) || q.phrase(&["was steht", "what is in"]) {
            return ok(format!("cat {}", sh(&n)), format!("Zeigt den Inhalt von „{n}“"));
        }
    }

    // ── Ordnerinhalt anzeigen ──
    if q.has(&["zeig", "list", "auflist", "anzeig", "inhalt", "was liegt", "was ist", "dateien", "umschau", "show", "display", "files", "contents"]) || q.phrase(&["was liegt", "was ist hier", "was gibt es", "what is here", "look around"]) {
        let hidden = q.has(&["versteckt", "unsichtbar", "alle", "alles", "hidden", "all", "invisible"]);
        let details = q.has(&["detail", "groesse", "datum", "genau", "liste", "rechte", "permission"]) || q.word(&["size", "sizes", "date", "dates"]);
        let newest = q.has(&["neueste", "neuesten", "zuletzt", "letzte", "aktuellste", "geaendert", "newest", "latest", "recent", "modified"]);
        let mut flags = String::new();
        if details || newest { flags.push('l'); }
        if hidden { flags.push('a'); }
        if details { flags.push('h'); }
        if newest { flags.push('t'); }
        let mut target = String::new();
        let mut where_ = "hier".to_string();
        if let Some((p, n)) = place(&q) {
            target = format!(" {p}");
            where_ = format!("in „{n}“");
        } else if let Some(n) = first.as_ref().filter(|n| is_dir(n)) {
            target = format!(" {}", sh(n));
            where_ = format!("in „{n}“");
        }
        let f = if flags.is_empty() { String::new() } else { format!(" -{flags}") };
        let mut text = format!("Zeigt, was {where_} liegt");
        if hidden { text.push_str(", auch versteckte Dateien"); }
        if details { text.push_str(", mit Details"); }
        if newest { text.push_str(", neueste zuerst"); }
        let mut cmd = format!("ls{f}{target}");
        if newest && !details {
            cmd.push_str(" | head -n 12");
        }
        return ok(cmd, text);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sandbox(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("kompass-nl-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("Fotos")).unwrap();
        std::fs::write(d.join("notiz.txt"), "x").unwrap();
        std::fs::write(d.join("Rechnung 2026.pdf"), "x").unwrap();
        d
    }

    fn cmd(text: &str, d: &Path) -> String {
        translate(text, d).map(|s| s.command).unwrap_or_else(|| "<nichts>".into())
    }

    #[test]
    fn erkennung() {
        assert_eq!(detect("zeige alle dateien", false).as_deref(), Some("zeige alle dateien"));
        assert_eq!(detect("? wie viel platz", true).as_deref(), Some("wie viel platz"));
        assert_eq!(detect("ls -la", true), None);
        assert_eq!(detect("./start.sh", false), None);
        assert_eq!(detect("wo bin ich?", false).as_deref(), Some("wo bin ich"));
    }

    #[test]
    fn grundlagen() {
        let d = sandbox("basis");
        assert_eq!(cmd("wo bin ich", &d), "pwd");
        assert_eq!(cmd("zeige alle dateien", &d), "ls -a");
        assert_eq!(cmd("zeig mir die versteckten dateien mit details", &d), "ls -lah");
        assert_eq!(cmd("was liegt im ordner fotos", &d), "ls Fotos");
        assert_eq!(cmd("geh in den ordner fotos", &d), "cd Fotos");
        assert_eq!(cmd("gehe nach hause", &d), "cd");
        assert_eq!(cmd("eine ebene nach oben", &d), "cd ..");
        assert_eq!(cmd("wechsle zum schreibtisch", &d), "cd ~/Desktop");
        assert_eq!(cmd("öffne den ordner im finder", &d), "open .");
    }

    #[test]
    fn dateien_bearbeiten() {
        let d = sandbox("dateien");
        assert_eq!(cmd("erstelle einen ordner namens Urlaub", &d), "mkdir -p Urlaub");
        assert_eq!(cmd("neue datei test.txt", &d), "touch test.txt");
        assert_eq!(cmd("lösche notiz.txt", &d), "rm notiz.txt");
        assert_eq!(cmd("lösche den ordner fotos", &d), "rm -r Fotos");
        assert_eq!(cmd("kopiere notiz.txt nach fotos", &d), "cp notiz.txt Fotos");
        assert_eq!(cmd("benenne notiz.txt in idee.txt um", &d), "mv notiz.txt idee.txt");
        assert_eq!(cmd("zeig den inhalt von notiz.txt", &d), "cat notiz.txt");
        assert_eq!(cmd("zeige die ersten 3 zeilen von notiz.txt", &d), "head -n 3 notiz.txt");
        assert_eq!(cmd("lösche „Rechnung 2026.pdf“", &d), "rm 'Rechnung 2026.pdf'");
        let s = translate("lösche eine datei", &d).unwrap();
        assert!(s.needs_input);
    }

    #[test]
    fn englisch() {
        crate::i18n::set_thread_en(true);
        let d = sandbox("englisch");
        assert_eq!(cmd("where am i", &d), "pwd");
        assert_eq!(cmd("show all files", &d), "ls -a");
        assert_eq!(cmd("find all pdf files", &d), "find . -type f -iname '*.pdf'");
        assert_eq!(cmd("go to the desktop", &d), "cd ~/Desktop");
        assert_eq!(cmd("create a folder called Holiday", &d), "mkdir -p Holiday");
        assert_eq!(cmd("delete notiz.txt", &d), "rm notiz.txt");
        assert_eq!(cmd("copy notiz.txt to fotos", &d), "cp notiz.txt Fotos");
        assert_eq!(cmd("rename notiz.txt to idea.txt", &d), "mv notiz.txt idea.txt");
        assert_eq!(cmd("how much disk space is free", &d), "df -h /");
        assert_eq!(cmd("which programs are running", &d), "top -o cpu -l 1 -n 10 -stats pid,command,cpu,mem");
        assert_eq!(cmd("open the downloads folder", &d), "open ~/Downloads");
        let s = translate("show all files", &d).unwrap();
        assert!(s.text.starts_with("Shows what is"), "{}", s.text);
        assert!(looks_like_sentence("find all pdf files"));
        assert!(looks_like_sentence("open the downloads folder"));
        assert!(!looks_like_sentence("find . -name x"));
        assert!(!looks_like_sentence("make install"));
        assert!(!looks_like_sentence("open notes.txt"));
        crate::i18n::set_thread_en(false);
    }

    #[test]
    fn suchen_und_system() {
        let d = sandbox("suchen");
        assert_eq!(cmd("finde alle pdf dateien", &d), "find . -type f -iname '*.pdf'");
        assert_eq!(cmd("such alle fotos auf dem schreibtisch", &d), "find ~/Desktop -type f \\( -iname '*.jpg' -o -iname '*.jpeg' -o -iname '*.png' -o -iname '*.heic' \\)");
        assert_eq!(cmd("suche nach \"Hallo\" in notiz.txt", &d), "grep -n Hallo notiz.txt");
        assert_eq!(cmd("wie viel speicherplatz ist frei", &d), "df -h /");
        assert_eq!(cmd("welche programme laufen gerade", &d), "top -o cpu -l 1 -n 10 -stats pid,command,cpu,mem");
        assert_eq!(cmd("wie spät ist es", &d), "date");
        assert_eq!(cmd("was ist meine ip adresse", &d), "ipconfig getifaddr en0");
        assert_eq!(cmd("packe den ordner fotos", &d), "zip -r Fotos.zip Fotos");
        assert_eq!(cmd("sag hallo welt", &d), "say 'hallo welt'");
        assert_eq!(cmd("wie groß ist der ordner fotos", &d), "du -sh Fotos");
        assert_eq!(cmd("zähle die dateien", &d), "ls | wc -l");
        assert_eq!(cmd("blubber wabbel", &d), "<nichts>");
    }
}
