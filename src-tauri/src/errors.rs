//! Fehler-Übersetzer: Ist ein Befehl schiefgegangen, sucht dieses Modul in
//! der Ausgabe nach bekannten (englischen) Fehlermeldungen und erklärt sie
//! auf Deutsch – wenn möglich mit einem Vorschlag, wie es richtig geht.

use crate::explain::{distance, suggest_command};
use crate::guard::{base, resolve};
use crate::i18n::tr;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct Help {
    pub title: String,
    pub text: String,
    /// Ein korrigierter Befehl, den Kompass per Knopfdruck einsetzen kann.
    pub suggestion: Option<String>,
}

/// `command` = getippte Zeile, `exit` = Rückgabewert (0 = Erfolg),
/// `output` = was der Befehl ausgegeben hat, `cwd` = Ordner nach dem Befehl.
pub fn analyze(command: &str, exit: i32, output: &str, cwd: &Path) -> Option<Help> {
    // 0 = alles gut; 130 = mit Ctrl+C abgebrochen; 141 = Pipe geschlossen (z. B. `| head`).
    if matches!(exit, 0 | 130 | 141) {
        return None;
    }
    let clean = strip_ansi(output);
    // Nur die letzten Zeilen ansehen – dort steht der Fehler.
    let lines: Vec<&str> = clean.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
    let tail = &lines[lines.len().saturating_sub(12)..];
    let first_word = command.split_whitespace().next().unwrap_or("");

    for line in tail.iter().rev() {
        let low = line.to_lowercase();

        if let Some(name) = after(line, &low, "command not found: ") {
            let suggestion = suggest_command(&name).map(|s| command.replacen(&name, s, 1));
            let text = match &suggestion {
                Some(s) => t!("„{name}“ gibt es nicht als Befehl. Vermutlich ein Tippfehler – meintest du „{}“?", "“{name}” is not a command. Probably a typo – did you mean “{}”?", s.split_whitespace().next().unwrap_or("")),
                None => t!("„{name}“ gibt es nicht als Befehl. Vielleicht ein Tippfehler – oder das Programm ist nicht installiert. Mit „which {name}“ prüfst du, ob es da ist.", "“{name}” is not a command. Maybe a typo – or the program is not installed. “which {name}” tells you whether it is there."),
            };
            return Some(Help { title: tr("Befehl unbekannt", "Unknown command"), text, suggestion });
        }

        if low.contains("no such file or directory") {
            let name = subject(line, &low, "no such file or directory").unwrap_or_default();
            let close = closest_entry(&name, cwd);
            let suggestion = close.as_ref().filter(|_| !name.is_empty()).map(|c| command.replacen(&name, c, 1));
            let mut text = if name.is_empty() {
                tr("Eine Datei oder ein Ordner wurde nicht gefunden.", "A file or folder was not found.")
            } else {
                t!("„{name}“ gibt es hier nicht.", "“{name}” does not exist here.")
            };
            match &close {
                Some(c) => text.push_str(&t!(" Es gibt aber „{c}“ – meintest du das? (Groß- und Kleinschreibung zählt!)", " But there is “{c}” – did you mean that? (Upper and lower case matter!)")),
                None => text.push_str(&tr(" Prüf die Schreibweise oder schau mit „ls“ nach, was hier liegt. Tipp: Mit der Tab-Taste ergänzt das Terminal Namen automatisch.", " Check the spelling or use “ls” to see what is here. Tip: the Tab key completes names automatically.")),
            }
            return Some(Help { title: tr("Nicht gefunden", "Not found"), text, suggestion });
        }

        if low.contains("operation not permitted") {
            return Some(Help {
                title: tr("macOS hat es blockiert", "Blocked by macOS"),
                text: tr("macOS schützt bestimmte Orte (z. B. Schreibtisch, Dokumente, Downloads, Papierkorb). Erlaube Kompass den Zugriff unter Systemeinstellungen → Datenschutz & Sicherheit → Dateien und Ordner bzw. Festplattenvollzugriff.", "macOS protects certain places (e.g. Desktop, Documents, Downloads, Trash). Allow Kompass access under System Settings → Privacy & Security → Files and Folders or Full Disk Access."),
                suggestion: None,
            });
        }

        if low.contains("permission denied") {
            let name = subject(line, &low, "permission denied").unwrap_or_default();
            let script = name.starts_with("./") || name.ends_with(".sh");
            let text = if script {
                t!("„{name}“ darf nicht als Programm gestartet werden – die Ausführ-Erlaubnis fehlt. Du kannst es stattdessen mit „zsh {name}“ starten.", "“{name}” may not be run as a program – it lacks execute permission. You can run it with “zsh {name}” instead.")
            } else {
                tr("Dafür fehlt dir die Berechtigung. Oft gehört die Datei einem anderen Benutzer oder dem System. Mit „ls -l“ siehst du, wem sie gehört.", "You lack the permission for this. Often the file belongs to another user or the system. “ls -l” shows who owns it.")
            };
            let suggestion = script.then(|| format!("zsh {name}"));
            return Some(Help { title: tr("Keine Berechtigung", "Permission denied"), text, suggestion });
        }

        if low.contains("is a directory") {
            let name = subject(line, &low, "is a directory").unwrap_or_default();
            return Some(Help {
                title: tr("Das ist ein Ordner", "That is a folder"),
                text: t!("„{name}“ ist ein Ordner, keine Datei. Mit „cd {name}“ gehst du hinein, mit „ls {name}“ siehst du, was darin liegt.", "“{name}” is a folder, not a file. “cd {name}” goes inside, “ls {name}” shows what is in it."),
                suggestion: (!name.is_empty()).then(|| format!("ls {name}")),
            });
        }

        if low.contains("not a directory") {
            let name = subject(line, &low, "not a directory").unwrap_or_default();
            return Some(Help {
                title: tr("Das ist kein Ordner", "That is not a folder"),
                text: t!("„{name}“ ist eine Datei, kein Ordner – hineingehen geht nicht. Mit „open {name}“ öffnest du sie.", "“{name}” is a file, not a folder – you cannot go inside. “open {name}” opens it."),
                suggestion: (!name.is_empty() && first_word == "cd").then(|| format!("open {name}")),
            });
        }

        if low.contains("directory not empty") {
            let name = subject(line, &low, "directory not empty").unwrap_or_default();
            return Some(Help {
                title: tr("Ordner ist nicht leer", "Folder is not empty"),
                text: t!("„{name}“ enthält noch etwas. „rmdir“ entfernt nur leere Ordner. Mit „rm -r {name}“ geht er samt Inhalt – in Kompass landet alles im Papierkorb.", "“{name}” still contains something. “rmdir” only removes empty folders. “rm -r {name}” removes it with its content – in Kompass everything goes to the Trash."),
                suggestion: (!name.is_empty()).then(|| format!("rm -r {name}")),
            });
        }

        if low.contains("file exists") {
            let name = subject(line, &low, "file exists").unwrap_or_default();
            return Some(Help {
                title: tr("Gibt es schon", "Already exists"),
                text: t!("„{name}“ existiert bereits. Nimm einen anderen Namen – oder bei Ordnern „mkdir -p“, das meckert dann nicht.", "“{name}” already exists. Pick another name – or for folders use “mkdir -p”, which does not complain."),
                suggestion: None,
            });
        }

        if let Some(pattern) = after(line, &low, "no matches found: ") {
            return Some(Help {
                title: tr("Muster passt auf nichts", "Pattern matches nothing"),
                text: t!("Für „{pattern}“ gibt es hier keine passenden Dateien. Prüf mit „ls“, wie die Dateien wirklich heißen.", "No files here match “{pattern}”. Use “ls” to check what the files are really called."),
                suggestion: Some("ls".into()),
            });
        }

        if low.contains("could not resolve host") || low.contains("unknown host") || low.contains("cannot resolve") {
            return Some(Help {
                title: tr("Adresse nicht gefunden", "Address not found"),
                text: tr("Die Internet-Adresse ließ sich nicht finden. Prüf die Schreibweise und ob du online bist.", "The internet address could not be found. Check the spelling and whether you are online."),
                suggestion: None,
            });
        }

        if low.contains("illegal option") || low.contains("invalid option") || low.contains("unrecognized option") || low.contains("unknown option") || low.starts_with("usage:") {
            let cmd = base(first_word);
            return Some(Help {
                title: tr("Falsche Angaben", "Invalid options"),
                text: t!("„{cmd}“ versteht diese Option oder Schreibweise nicht. In der Anleitung steht, was geht – „q“ schließt sie wieder.", "“{cmd}” does not understand this option or syntax. The manual lists what works – “q” closes it again."),
                suggestion: (!cmd.is_empty()).then(|| format!("man {cmd}")),
            });
        }
    }
    None
}

/// Text hinter `marker` (Groß/Klein egal), z. B. „command not found: sl“ → „sl“.
fn after(line: &str, low: &str, marker: &str) -> Option<String> {
    // Byte-Positionen in `low` und `line` stimmen überein, solange nur ASCII-Buchstaben
    // betroffen sind – die Meldungen sind englisch, daher passt das.
    let pos = low.find(marker)?;
    let rest = line.get(pos + marker.len()..)?.trim();
    (!rest.is_empty()).then(|| rest.to_string())
}

/// Der Name, um den es in einer Meldung geht. Zwei Formen sind üblich:
///   „cd: no such file or directory: Fotoss“   → Name steht dahinter
///   „ls: Fotoss: No such file or directory“   → Name steht davor
fn subject(line: &str, low: &str, marker: &str) -> Option<String> {
    if let Some(s) = after(line, low, &format!("{marker}: ")) {
        return Some(s);
    }
    let pos = low.find(marker)?;
    let before = line.get(..pos)?.trim_end().trim_end_matches(':').trim_end();
    // Ohne „: “ steht davor nur der Programmname (z. B. „zsh“) – dann kein Name.
    if !before.contains(": ") {
        return None;
    }
    // Letzter Abschnitt nach „: “, davor steht der Programmname.
    let name = before.rsplit(": ").next()?.trim();
    (!name.is_empty()).then(|| name.to_string())
}

/// Sucht im Ordner einen ähnlich geschriebenen Eintrag (für „meintest du …?“).
fn closest_entry(name: &str, cwd: &Path) -> Option<String> {
    if name.is_empty() {
        return None;
    }
    // Bei `Fotos/Urlaub` im Ordner `Fotos` suchen.
    let path = Path::new(name);
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
    let dir = match parent {
        Some(p) => resolve(&p.to_string_lossy(), cwd),
        None => cwd.to_path_buf(),
    };
    let wanted = path.file_name()?.to_string_lossy().to_lowercase();
    let best = std::fs::read_dir(&dir)
        .ok()?
        .take(5000)
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .map(|n| {
            let d = distance(&wanted, &n.to_lowercase());
            (n, d)
        })
        .filter(|(n, d)| *d <= if n.len() <= 4 { 1 } else { 2 })
        .min_by_key(|(_, d)| *d)?
        .0;
    Some(match parent {
        Some(p) => format!("{}/{}", p.to_string_lossy(), best),
        None => best,
    })
}

/// Entfernt Farb- und Steuercodes (ESC [ … Buchstabe, ESC ] … BEL) aus der Ausgabe.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.next() {
                Some('[') => {
                    // bis zum ersten Buchstaben überspringen
                    while let Some(n) = chars.next() {
                        if n.is_ascii_alphabetic() || n == '~' { break; }
                    }
                }
                Some(']') => {
                    while let Some(n) = chars.next() {
                        if n == '\x07' { break; }
                        if n == '\x1b' { chars.next(); break; }
                    }
                }
                _ => {}
            }
        } else if c == '\r' || c == '\x08' {
            // Wagenrücklauf/Rücktaste ignorieren
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Jeder Test bekommt einen eigenen Ordner – Tests laufen parallel.
    fn sandbox(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("kompass-errors-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("Fotos")).unwrap();
        std::fs::write(d.join("notiz.txt"), "x").unwrap();
        d
    }

    #[test]
    fn tippfehler_beim_befehl() {
        let h = analyze("sl -la", 127, "zsh: command not found: sl\r\n", &sandbox("tippfehler_beim_befehl")).unwrap();
        assert_eq!(h.title, "Befehl unbekannt");
        assert_eq!(h.suggestion.as_deref(), Some("ls -la"));
    }

    #[test]
    fn datei_nicht_gefunden_beide_formen() {
        let d = sandbox("datei_nicht_gefunden_beide_formen");
        let h = analyze("cd fotos", 1, "cd: no such file or directory: fotos\r\n", &d).unwrap();
        assert_eq!(h.suggestion.as_deref(), Some("cd Fotos"));
        let h = analyze("cat notitz.txt", 1, "\x1b[0mcat: notitz.txt: No such file or directory\r\n", &d).unwrap();
        assert!(h.text.contains("„notitz.txt“ gibt es hier nicht"), "{}", h.text);
        assert_eq!(h.suggestion.as_deref(), Some("cat notiz.txt"));
    }

    #[test]
    fn weitere_fehler() {
        let d = sandbox("weitere_fehler");
        assert_eq!(analyze("cat Fotos", 1, "cat: Fotos: Is a directory", &d).unwrap().suggestion.as_deref(), Some("ls Fotos"));
        assert_eq!(analyze("rmdir Fotos", 1, "rmdir: Fotos: Directory not empty", &d).unwrap().title, "Ordner ist nicht leer");
        assert_eq!(analyze("./x.sh", 126, "zsh: permission denied: ./x.sh", &d).unwrap().suggestion.as_deref(), Some("zsh ./x.sh"));
        assert_eq!(analyze("ls -y", 1, "ls: invalid option -- y\nusage: ls [-@ABC]", &d).unwrap().suggestion.as_deref(), Some("man ls"));
        assert!(analyze("ls", 0, "", &d).is_none());
        assert!(analyze("sleep 5", 130, "^C", &d).is_none());
    }
}
