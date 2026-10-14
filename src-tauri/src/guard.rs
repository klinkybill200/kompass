//! Der Befehls-Wächter.
//!
//! Bekommt eine eingegebene Zeile (z. B. `cd Fotos && rm *.jpg`) plus den
//! aktuellen Ordner, zerlegt sie in Einzelbefehle und stuft sie ein:
//! sicher, aufpassen oder gefährlich – jeweils mit einer Erklärung in Klartext.

use crate::i18n::tr;
use serde::Serialize;
use std::path::{Path, PathBuf};

// ───────────────────────────── Ergebnis-Typen ─────────────────────────────

/// Die drei Stufen. Durch `PartialOrd, Ord` kann Rust sie vergleichen:
/// Safe < Care < Danger (Reihenfolge wie hier aufgeschrieben).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Level {
    Safe,
    Care,
    Danger,
}

/// Eine einzelne Beobachtung, z. B. „Löscht 3 Dateien“.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub level: Level,
    pub title: String,
    pub text: String,
}

/// Eine Datei bzw. ein Ordner, den ein Befehl betrifft (für die Anzeige im Dialog).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub name: String,
    /// "file", "dir", "missing" oder "unknown"
    pub kind: &'static str,
    /// Bei Ordnern: wie viele Einträge direkt darin liegen.
    pub entries: Option<usize>,
}

/// Plan für „rm → Papierkorb“: der umgeschriebene Befehl und die betroffenen Dateien.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashPlan {
    pub rewritten: String,
    pub targets: Vec<Target>,
}

/// Das Gesamturteil über eine Zeile.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Verdict {
    pub level: Level,
    pub findings: Vec<Finding>,
    /// `Some(...)` wenn rm erkannt wurde und in den Papierkorb umgeleitet werden kann.
    pub trash: Option<TrashPlan>,
}

// ───────────────────────────── Zerlegen (Tokenizer) ─────────────────────────────

/// Ein Wort der Befehlszeile, z. B. `'mein ordner'` → Text `mein ordner`.
#[derive(Debug, Clone)]
// `pub(crate)` = innerhalb von Kompass sichtbar (explain.rs nutzt den Zerleger mit).
pub(crate) struct Word {
    pub(crate) text: String,  // ohne Anführungszeichen
    pub(crate) start: usize,  // Position im Original (Byte-Index) – zum Umschreiben
    pub(crate) end: usize,
    pub(crate) glob: bool,    // enthält *, ? oder [ ohne Anführungszeichen
    pub(crate) dynamic: bool, // enthält $VARIABLE oder $(befehl) – Wert erst zur Laufzeit bekannt
}

/// Bausteine der Zeile: Wörter, Verknüpfungen (`&&`, `|`, `;` …) und Umleitungen (`>`, `>>` …).
#[derive(Debug, Clone)]
pub(crate) enum Tok {
    Word(Word),
    Op(String),
    Redir(String),
}

fn is_separator(c: char) -> bool {
    c.is_whitespace() || matches!(c, ';' | '&' | '|' | '(' | ')' | '<' | '>')
}

/// Zerlegt die Zeile ähnlich wie die zsh – vereinfacht, aber Anführungszeichen,
/// Backslashes, `$(...)`, Verknüpfungen und Umleitungen werden verstanden.
pub(crate) fn tokenize(line: &str) -> Vec<Tok> {
    // (Byte-Position, Zeichen)-Paare; so können wir Spannen im Original merken.
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let at = |i: usize| chars.get(i).map(|&(_, c)| c); // Zeichen an Stelle i (oder None)
    let pos = |i: usize| chars.get(i).map(|&(p, _)| p).unwrap_or(line.len());
    let mut toks = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i].1;
        // `match` ist wie ein mächtiges switch/case.
        match c {
            c if c == '\n' || c == ';' => {
                toks.push(Tok::Op(";".into()));
                i += 1;
            }
            c if c.is_whitespace() => i += 1,
            '&' => {
                if at(i + 1) == Some('&') {
                    toks.push(Tok::Op("&&".into()));
                    i += 2;
                } else if at(i + 1) == Some('>') {
                    // `&>` bzw. `&>>`: Ausgabe UND Fehler in Datei
                    let op = if at(i + 2) == Some('>') { "&>>" } else { "&>" };
                    i += op.len();
                    toks.push(Tok::Redir(op.into()));
                } else {
                    toks.push(Tok::Op("&".into()));
                    i += 1;
                }
            }
            '|' => {
                if at(i + 1) == Some('|') {
                    toks.push(Tok::Op("||".into()));
                    i += 2;
                } else {
                    toks.push(Tok::Op("|".into()));
                    i += if at(i + 1) == Some('&') { 2 } else { 1 };
                }
            }
            '(' | ')' => {
                toks.push(Tok::Op(c.to_string()));
                i += 1;
            }
            '<' | '>' => {
                let (op, len) = read_redir(&chars, i);
                toks.push(Tok::Redir(op));
                i += len;
            }
            _ => {
                let start = i;
                let mut w = Word { text: String::new(), start: pos(i), end: 0, glob: false, dynamic: false };
                let mut quoted = false;
                while let Some(c) = at(i) {
                    if is_separator(c) {
                        break;
                    }
                    match c {
                        '\'' => {
                            quoted = true;
                            i += 1;
                            while let Some(c) = at(i) {
                                i += 1;
                                if c == '\'' { break; }
                                w.text.push(c);
                            }
                        }
                        '"' => {
                            quoted = true;
                            i += 1;
                            while let Some(c) = at(i) {
                                i += 1;
                                match c {
                                    '"' => break,
                                    '\\' => {
                                        if let Some(n) = at(i) {
                                            w.text.push(n);
                                            i += 1;
                                        }
                                    }
                                    '$' | '`' => {
                                        w.dynamic = true;
                                        w.text.push(c);
                                    }
                                    _ => w.text.push(c),
                                }
                            }
                        }
                        '\\' => {
                            if let Some(n) = at(i + 1) {
                                w.text.push(n);
                            }
                            i += 2;
                        }
                        '$' if at(i + 1) == Some('(') => {
                            // $( ... ) bis zur passenden Klammer überspringen
                            w.dynamic = true;
                            let mut depth = 0;
                            while let Some(c) = at(i) {
                                w.text.push(c);
                                i += 1;
                                if c == '(' { depth += 1; }
                                if c == ')' {
                                    depth -= 1;
                                    if depth == 0 { break; }
                                }
                            }
                        }
                        '$' | '`' => {
                            w.dynamic = true;
                            w.text.push(c);
                            i += 1;
                        }
                        '*' | '?' | '[' => {
                            w.glob = true;
                            w.text.push(c);
                            i += 1;
                        }
                        _ => {
                            w.text.push(c);
                            i += 1;
                        }
                    }
                }
                w.end = pos(i);
                // Sonderfall `2>datei`: die Ziffer gehört zur Umleitung, ist kein Wort.
                let digits_only = !quoted && !w.text.is_empty() && w.text.chars().all(|c| c.is_ascii_digit());
                if digits_only && i == start + w.text.len() && matches!(at(i), Some('<' | '>')) {
                    let (op, len) = read_redir(&chars, i);
                    toks.push(Tok::Redir(format!("{}{}", w.text, op)));
                    i += len;
                } else {
                    toks.push(Tok::Word(w));
                }
            }
        }
    }
    toks
}

/// Liest eine Umleitung wie `>`, `>>`, `>|`, `<`, `<<`, `<<<`, `>&` ab Position i.
fn read_redir(chars: &[(usize, char)], i: usize) -> (String, usize) {
    let mut op = String::new();
    let mut j = i;
    while let Some(&(_, c)) = chars.get(j) {
        let ok = match op.as_str() {
            "" => c == '<' || c == '>',
            ">" => c == '>' || c == '|' || c == '&',
            "<" => c == '<' || c == '>' || c == '&',
            "<<" => c == '<',
            _ => false,
        };
        if !ok { break; }
        op.push(c);
        j += 1;
    }
    (op, j - i)
}

/// Ein einzelner Befehl innerhalb der Zeile (zwischen `;`, `&&`, `|` …).
#[derive(Debug, Default)]
pub(crate) struct Segment {
    pub(crate) words: Vec<Word>,
    pub(crate) redirs: Vec<(String, Option<Word>)>, // (Art, Ziel)
    pub(crate) piped_from_prev: bool,               // steht hinter einem `|`
    pub(crate) op_before: Option<String>,           // Verknüpfung davor: `&&`, `|`, `;` …
}

pub(crate) fn split_segments(toks: Vec<Tok>) -> Vec<Segment> {
    let mut segs = vec![Segment::default()];
    let mut iter = toks.into_iter().peekable();
    // `while let Some(t) = iter.next()` = „solange es noch Bausteine gibt“
    while let Some(t) = iter.next() {
        let cur = segs.last_mut().unwrap();
        match t {
            Tok::Word(w) => cur.words.push(w),
            Tok::Redir(op) => {
                // Das Ziel ist das nächste Wort (z. B. `> notizen.txt`).
                let target = match iter.peek() {
                    Some(Tok::Word(_)) => match iter.next() {
                        Some(Tok::Word(w)) => Some(w),
                        _ => None,
                    },
                    _ => None,
                };
                cur.redirs.push((op, target));
            }
            Tok::Op(op) => {
                segs.push(Segment { piped_from_prev: op == "|", op_before: Some(op), ..Default::default() });
            }
        }
    }
    // Leere Segmente (z. B. durch `( ... )`) entfernen.
    segs.into_iter().filter(|s| !s.words.is_empty() || !s.redirs.is_empty()).collect()
}

// ───────────────────────────── Bewerten ─────────────────────────────

/// Hauptfunktion: bewertet eine komplette Zeile im Ordner `cwd`.
pub fn check(line: &str, cwd: &Path) -> Verdict {
    let segments = split_segments(tokenize(line));
    let mut findings: Vec<Finding> = Vec::new();
    // Stellen im Original, an denen `rm` durch `kompass-trash` ersetzt wird.
    let mut rewrites: Vec<(usize, usize)> = Vec::new();
    let mut trash_targets: Vec<Target> = Vec::new();
    let mut rm_blocks_trash = false; // z. B. `sudo rm` – geht nicht in den Papierkorb
    let mut prev_cmd = String::new();

    for seg in &segments {
        let ctx = Ctx { cwd, seg, prev_cmd: &prev_cmd };
        let cmd = analyze_segment(&ctx, &mut findings, &mut rewrites, &mut trash_targets, &mut rm_blocks_trash);
        prev_cmd = cmd;
        check_redirects(seg, cwd, &mut findings);
    }

    // Doppelte Meldungen entfernen (z. B. zweimal dieselbe Umleitung).
    let mut seen = std::collections::HashSet::new();
    findings.retain(|f| seen.insert((f.title.clone(), f.text.clone())));

    let level = findings.iter().map(|f| f.level).max().unwrap_or(Level::Safe);

    let trash = if !rewrites.is_empty() && !rm_blocks_trash {
        // Von hinten nach vorn ersetzen, damit sich die Positionen nicht verschieben.
        let mut rewritten = line.to_string();
        rewrites.sort();
        for (s, e) in rewrites.iter().rev() {
            rewritten.replace_range(*s..*e, "kompass-trash");
        }
        Some(TrashPlan { rewritten, targets: trash_targets })
    } else {
        None
    };

    Verdict { level, findings, trash }
}

/// Alles, was die Bewertung eines Einzelbefehls wissen muss.
/// `'a` ist eine „Lebensdauer“: die Referenzen leben mindestens so lange wie Ctx.
struct Ctx<'a> {
    cwd: &'a Path,
    seg: &'a Segment,
    prev_cmd: &'a str,
}

fn finding(level: Level, title: impl Into<String>, text: impl Into<String>) -> Finding {
    Finding { level, title: title.into(), text: text.into() }
}

/// Dateiname ohne Pfad: `/bin/rm` → `rm`.
pub(crate) fn base(cmd: &str) -> &str {
    cmd.rsplit('/').next().unwrap_or(cmd)
}

pub(crate) fn is_assignment(w: &Word) -> bool {
    match w.text.split_once('=') {
        Some((name, _)) => {
            !name.is_empty()
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && !name.starts_with(|c: char| c.is_ascii_digit())
        }
        None => false,
    }
}

/// Bewertet einen Einzelbefehl. Gibt den Befehlsnamen zurück (für Pipes wie `curl … | sh`).
fn analyze_segment(
    ctx: &Ctx,
    out: &mut Vec<Finding>,
    rewrites: &mut Vec<(usize, usize)>,
    trash_targets: &mut Vec<Target>,
    rm_blocks_trash: &mut bool,
) -> String {
    let words = &ctx.seg.words;
    // Variablen-Zuweisungen am Anfang (`LANG=C ls`) überspringen.
    let mut i = 0;
    while i < words.len() && is_assignment(&words[i]) {
        i += 1;
    }
    let chain_start = i; // ab hier wird bei rm ersetzt
    let mut via_sudo = false;
    let mut via_wrapper = false; // `command`, `exec`, `nohup` … rufen das echte rm auf

    // Vorangestellte Helfer wie `sudo`, `command`, `time` auswerten.
    while i < words.len() {
        match base(&words[i].text) {
            "sudo" | "doas" => {
                via_sudo = true;
                out.push(finding(
                    Level::Danger,
                    tr("Mit Administrator-Rechten", "With administrator rights"),
                    tr("„sudo“ führt den Befehl als Administrator aus. Er darf dann alles auf deinem Mac verändern – auch Systemdateien. Du wirst nach deinem Mac-Passwort gefragt.", "“sudo” runs the command as administrator. It may then change anything on your Mac – including system files. You will be asked for your Mac password."),
                ));
                i += 1;
                // Optionen von sudo überspringen (`-u benutzer` hat ein Argument).
                while i < words.len() && words[i].text.starts_with('-') {
                    let takes_arg = matches!(words[i].text.as_str(), "-u" | "-g" | "-C" | "-h" | "-p" | "-U");
                    i += if takes_arg { 2 } else { 1 };
                }
            }
            "command" | "builtin" | "exec" | "nohup" | "env" | "xargs" => {
                via_wrapper = true;
                i += 1;
                while i < words.len() && (words[i].text.starts_with('-') || is_assignment(&words[i])) {
                    i += 1;
                }
            }
            "time" | "noglob" | "nocorrect" => i += 1,
            _ => break,
        }
    }

    // `words.get(i)` liefert None, wenn es kein Wort an Stelle i gibt.
    let Some(cmd_word) = words.get(i) else {
        return String::new();
    };
    let cmd = base(&cmd_word.text).to_string();
    let args: Vec<&Word> = words[i + 1..].iter().collect();
    let flags = |s: &str| has_flag(&args, s);

    match cmd.as_str() {
        "rm" | "unlink" | "srm" => {
            let targets = collect_targets(&args, ctx.cwd);
            let recursive = flags("r") || flags("R");
            let names = describe_targets(&targets);
            let mut text = t!("Löscht {names}.", "Deletes {names}.");
            if recursive && targets.iter().any(|t| t.kind == "dir") {
                text.push_str(&tr(" Ordner werden samt ihrem kompletten Inhalt gelöscht.", " Folders are deleted together with everything inside."));
            }
            if via_sudo || via_wrapper || cmd == "srm" {
                *rm_blocks_trash = true;
                text.push_str(&tr(" Achtung: Das geht nicht über den Papierkorb – gelöschte Dateien sind endgültig weg.", " Careful: this does not go through the Trash – deleted files are gone for good."));
            } else {
                text.push_str(&tr(" Normalerweise ist das endgültig. Kompass legt die Dateien stattdessen in den Papierkorb – du kannst sie jederzeit zurückholen.", " Normally this is permanent. Kompass moves the files to the Trash instead – you can get them back at any time."));
                rewrites.push((words[chain_start].start, cmd_word.end));
                trash_targets.extend(targets);
            }
            out.push(finding(Level::Danger, tr("Dateien löschen", "Delete files"), text));
        }
        "rmdir" => {
            let targets = collect_targets(&args, ctx.cwd);
            out.push(finding(
                Level::Care,
                tr("Leere Ordner entfernen", "Remove empty folders"),
                t!("Entfernt {} – das klappt nur, wenn die Ordner leer sind.", "Removes {} – this only works if the folders are empty.", describe_targets(&targets)),
            ));
        }
        "chmod" => {
            let files: Vec<&Word> = args.iter().copied().filter(|w| !w.text.starts_with('-')).skip(1).collect();
            let targets = collect_targets(&files, ctx.cwd);
            let mut text = t!("Ändert, wer {} lesen, verändern oder ausführen darf.", "Changes who may read, change or run {}.", describe_targets(&targets));
            if flags("R") {
                text.push_str(&tr(" Gilt auch für alles, was in den Ordnern liegt.", " Also applies to everything inside the folders."));
            }
            text.push_str(&tr(" Falsche Rechte können Programme oder deinen Zugriff auf Dateien blockieren.", " Wrong permissions can break programs or lock you out of your files."));
            out.push(finding(Level::Danger, tr("Zugriffsrechte ändern", "Change permissions"), text));
        }
        "chown" | "chgrp" => {
            let files: Vec<&Word> = args.iter().copied().filter(|w| !w.text.starts_with('-')).skip(1).collect();
            let targets = collect_targets(&files, ctx.cwd);
            out.push(finding(
                Level::Danger,
                tr("Besitzer ändern", "Change owner"),
                t!("Ändert, wem {} gehört. Danach kannst du sie unter Umständen nicht mehr öffnen oder ändern.", "Changes who owns {}. Afterwards you may no longer be able to open or change them.", describe_targets(&targets)),
            ));
        }
        "dd" => out.push(finding(
            Level::Danger,
            tr("Rohdaten schreiben", "Write raw data"),
            tr("„dd“ kopiert Daten Byte für Byte – auch direkt auf ganze Laufwerke. Ein Tippfehler beim Ziel (of=…) kann eine Festplatte unwiderruflich überschreiben.", "“dd” copies data byte by byte – even straight onto whole drives. A typo in the target (of=…) can overwrite a disk irreversibly."),
        )),
        "mkfs" | "newfs_apfs" | "newfs_hfs" => out.push(finding(
            Level::Danger,
            tr("Laufwerk formatieren", "Format a drive"),
            tr("Formatiert ein Laufwerk. Alles, was darauf gespeichert ist, wird gelöscht.", "Formats a drive. Everything stored on it will be erased."),
        )),
        "diskutil" => {
            let sub = args.first().map(|w| w.text.to_lowercase()).unwrap_or_default();
            if sub.starts_with("erase") || sub.starts_with("partition") || sub.starts_with("zero") || sub.starts_with("secureerase") || sub == "apfs" {
                out.push(finding(
                    Level::Danger,
                    tr("Laufwerk löschen", "Erase a drive"),
                    tr("Dieser diskutil-Befehl löscht oder partitioniert ein Laufwerk. Alles, was darauf gespeichert ist, geht verloren.", "This diskutil command erases or partitions a drive. Everything stored on it will be lost."),
                ));
            }
        }
        "shutdown" | "reboot" | "halt" => out.push(finding(
            Level::Danger,
            tr("Mac ausschalten", "Shut down the Mac"),
            tr("Fährt den Mac herunter oder startet ihn neu. Ungespeicherte Arbeit in allen Programmen geht verloren.", "Shuts down or restarts the Mac. Unsaved work in all programs will be lost."),
        )),
        "kill" | "killall" | "pkill" => out.push(finding(
            Level::Care,
            tr("Programm beenden", "Quit a program"),
            tr("Beendet ein laufendes Programm sofort. Ungespeicherte Arbeit darin geht verloren.", "Quits a running program immediately. Unsaved work in it will be lost."),
        )),
        "mv" | "cp" => {
            let verb = if cmd == "mv" { tr("Verschiebt", "Moves") } else { tr("Kopiert", "Copies") };
            let ops: Vec<&Word> = args.iter().copied().filter(|w| !w.text.starts_with('-')).collect();
            if let Some((dest, sources)) = ops.split_last() {
                let overwritten = overwritten_by_copy(sources, dest, ctx.cwd);
                if !overwritten.is_empty() && !flags("n") && !flags("i") {
                    out.push(finding(
                        Level::Care,
                        tr("Überschreibt Dateien", "Overwrites files"),
                        t!(
                            "{verb} nach „{}“. Dabei wird {} überschrieben – der alte Inhalt ist danach weg.",
                            "{verb} to “{}”. This overwrites {} – the old content will be gone.",
                            dest.text,
                            quote_list(&overwritten)
                        ),
                    ));
                }
            }
        }
        "find" => {
            let has_delete = args.iter().any(|w| w.text == "-delete");
            let exec_rm = args.windows(2).any(|p| (p[0].text == "-exec" || p[0].text == "-execdir") && base(&p[1].text) == "rm");
            if has_delete || exec_rm {
                out.push(finding(
                    Level::Danger,
                    tr("Suchen und löschen", "Find and delete"),
                    tr("Dieser find-Befehl löscht alle gefundenen Dateien endgültig – ohne Papierkorb. Probier ihn zuerst ohne -delete bzw. -exec aus, dann siehst du, was er finden würde.", "This find command permanently deletes every file it finds – without the Trash. Try it without -delete or -exec first to see what it would find."),
                ));
            }
        }
        "git" => {
            let a: Vec<&str> = args.iter().map(|w| w.text.as_str()).collect();
            let sub = a.first().copied().unwrap_or("");
            let has = |x: &str| a.contains(&x);
            let destructive = (sub == "reset" && has("--hard"))
                || (sub == "clean" && a.iter().any(|x| x.starts_with('-') && x.contains('f')))
                || (sub == "push" && (has("--force") || has("-f")))
                || (sub == "checkout" && has("."))
                || (sub == "restore" && !has("--staged"))
                || (sub == "branch" && has("-D"));
            if destructive {
                out.push(finding(
                    Level::Danger,
                    tr("Änderungen verwerfen", "Discard changes"),
                    tr("Dieser git-Befehl verwirft Änderungen oder überschreibt Verlauf. Was nicht gespeichert (committet) ist, lässt sich danach nicht wiederherstellen.", "This git command discards changes or rewrites history. Anything not committed cannot be recovered afterwards."),
                ));
            }
        }
        "brew" | "npm" | "pip" | "pip3" | "gem" | "port" => {
            let sub = args.first().map(|w| w.text.as_str()).unwrap_or("");
            if matches!(sub, "install" | "i" | "uninstall" | "remove" | "upgrade" | "update") {
                out.push(finding(
                    Level::Care,
                    tr("Software installieren", "Install software"),
                    t!("Installiert, entfernt oder aktualisiert Software über „{cmd}“. Das lädt Programme aus dem Internet.", "Installs, removes or updates software via “{cmd}”. This downloads programs from the internet."),
                ));
            }
        }
        "sh" | "bash" | "zsh" | "python" | "python3" | "ruby" | "perl" | "node"
            if ctx.seg.piped_from_prev && matches!(ctx.prev_cmd, "curl" | "wget") =>
        {
            out.push(finding(
                Level::Danger,
                tr("Skript aus dem Internet ausführen", "Run a script from the internet"),
                tr("Lädt ein Skript aus dem Internet und führt es sofort aus – ohne dass du vorher siehst, was es tut. Mach das nur, wenn du der Quelle vertraust.", "Downloads a script from the internet and runs it right away – without you seeing what it does. Only do this if you trust the source."),
            ));
        }
        "curl" | "wget" => {
            if cmd == "wget" || flags("o") || flags("O") || args.iter().any(|w| w.text == "--output") {
                out.push(finding(Level::Care, tr("Datei herunterladen", "Download a file"), tr("Lädt eine Datei aus dem Internet herunter und speichert sie hier.", "Downloads a file from the internet and saves it here.")));
            }
        }
        _ => {}
    }
    cmd
}

/// Prüft `>`-Umleitungen: Überschreiben einer bestehenden Datei ist gefährlich.
fn check_redirects(seg: &Segment, cwd: &Path, out: &mut Vec<Finding>) {
    for (op, target) in &seg.redirs {
        let overwrites = matches!(op.trim_start_matches(|c: char| c.is_ascii_digit()), ">" | ">|" | "&>");
        if !overwrites {
            continue; // `>>` hängt nur an, `<` liest nur
        }
        let Some(t) = target else { continue };
        if t.text.starts_with("/dev/") {
            continue; // z. B. `> /dev/null` = Ausgabe wegwerfen
        }
        if t.dynamic || t.glob {
            out.push(finding(
                Level::Care,
                tr("In Datei schreiben", "Write to a file"),
                t!("Schreibt die Ausgabe in „{}“. Kompass kann vorher nicht prüfen, ob es diese Datei schon gibt.", "Writes the output to “{}”. Kompass cannot check beforehand whether this file already exists.", t.text),
            ));
            continue;
        }
        let path = resolve(&t.text, cwd);
        if path.is_file() {
            out.push(finding(
                Level::Danger,
                tr("Datei überschreiben", "Overwrite a file"),
                t!(
                    "Das „>“ ersetzt den kompletten Inhalt von „{}“ – der bisherige Inhalt ist danach weg. Willst du stattdessen etwas anhängen, nimm „>>“.",
                    "The “>” replaces the entire content of “{}” – the previous content will be gone. To append instead, use “>>”.",
                    t.text
                ),
            ));
        }
    }
}

// ───────────────────────────── Hilfsfunktionen ─────────────────────────────

/// Ist eine Kurz-Option gesetzt? `-rf` enthält sowohl `r` als auch `f`.
fn has_flag(args: &[&Word], letter: &str) -> bool {
    args.iter().any(|w| {
        let t = &w.text;
        t.starts_with('-') && !t.starts_with("--") && t[1..].contains(letter)
    })
}

/// `~/x` und relative Pfade in einen vollen Pfad umwandeln.
pub(crate) fn resolve(p: &str, cwd: &Path) -> PathBuf {
    if p == "~" || p.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(p.trim_start_matches('~').trim_start_matches('/'));
        }
    }
    cwd.join(p) // ist p schon absolut, ersetzt `join` den Pfad einfach
}

/// Sammelt die Dateien, die ein Befehl wie rm betrifft – inkl. `*.jpg`-Mustern.
fn collect_targets(args: &[&Word], cwd: &Path) -> Vec<Target> {
    let mut out = Vec::new();
    let mut opts_done = false;
    for w in args {
        if !opts_done && w.text == "--" {
            opts_done = true;
            continue;
        }
        if !opts_done && w.text.starts_with('-') && w.text.len() > 1 {
            continue;
        }
        if w.dynamic {
            out.push(Target { name: w.text.clone(), kind: "unknown", entries: None });
        } else if w.glob {
            // Muster wie `*.jpg` selbst auflösen, um zu zeigen, was betroffen ist.
            let base = glob::Pattern::escape(&cwd.to_string_lossy());
            let pattern = if w.text.starts_with('/') { w.text.clone() } else { format!("{base}/{}", w.text) };
            let mut found = false;
            if let Ok(paths) = glob::glob(&pattern) {
                // `flatten()` überspringt fehlerhafte Einträge
                for p in paths.flatten() {
                    found = true;
                    let name = p.strip_prefix(cwd).map(|r| r.to_string_lossy().into_owned()).unwrap_or_else(|_| p.to_string_lossy().into_owned());
                    out.push(describe_path(name, &p));
                }
            }
            if !found {
                out.push(Target { name: w.text.clone(), kind: "missing", entries: None });
            }
        } else {
            out.push(describe_path(w.text.clone(), &resolve(&w.text, cwd)));
        }
    }
    out
}

fn describe_path(name: String, p: &Path) -> Target {
    // `symlink_metadata` folgt Verknüpfungen nicht – eine Verknüpfung zählt als Datei.
    match std::fs::symlink_metadata(p) {
        Ok(m) if m.is_dir() => {
            let entries = std::fs::read_dir(p).ok().map(|d| d.take(10_000).count());
            Target { name, kind: "dir", entries }
        }
        Ok(_) => Target { name, kind: "file", entries: None },
        Err(_) => Target { name, kind: "missing", entries: None },
    }
}

/// Fasst Ziele in einen lesbaren Satzteil, z. B. „die Datei „a.txt““ oder „3 Dateien und 1 Ordner“.
fn describe_targets(t: &[Target]) -> String {
    let files = t.iter().filter(|x| x.kind == "file").count();
    let dirs = t.iter().filter(|x| x.kind == "dir").count();
    let other = t.len() - files - dirs;
    if t.is_empty() {
        return tr("nichts (keine Dateien angegeben)", "nothing (no files given)");
    }
    if t.len() == 1 {
        let x = &t[0];
        return match x.kind {
            "file" => t!("die Datei „{}“", "the file “{}”", x.name),
            "dir" => t!("den Ordner „{}“", "the folder “{}”", x.name),
            "missing" => t!("„{}“ (gibt es hier aber nicht)", "“{}” (which does not exist here)", x.name),
            _ => t!("„{}“", "“{}”", x.name),
        };
    }
    let mut parts = Vec::new();
    if files > 0 { parts.push(if files == 1 { tr("1 Datei", "1 file") } else { t!("{files} Dateien", "{files} files") }); }
    if dirs > 0 { parts.push(if dirs == 1 { tr("1 Ordner", "1 folder") } else { t!("{dirs} Ordner", "{dirs} folders") }); }
    if other > 0 { parts.push(t!("{other} weitere Einträge", "{other} other entries")); }
    parts.join(&tr(" und ", " and "))
}

/// Welche bestehenden Dateien würde `cp`/`mv` überschreiben?
fn overwritten_by_copy(sources: &[&Word], dest: &Word, cwd: &Path) -> Vec<String> {
    if dest.dynamic || dest.glob {
        return Vec::new();
    }
    let d = resolve(&dest.text, cwd);
    if d.is_dir() {
        // `cp a.txt ordner/` → überschreibt `ordner/a.txt`, falls vorhanden
        sources
            .iter()
            .filter_map(|s| {
                let name = Path::new(&s.text).file_name()?.to_string_lossy().into_owned();
                d.join(&name).is_file().then(|| format!("{}/{}", dest.text.trim_end_matches('/'), name))
            })
            .collect()
    } else if d.is_file() {
        vec![dest.text.clone()]
    } else {
        Vec::new()
    }
}

fn quote_list(items: &[String]) -> String {
    let q: Vec<String> = items.iter().take(5).map(|s| t!("„{s}“", "“{s}”")).collect();
    let more = items.len().saturating_sub(5);
    if more > 0 { t!("{} und {more} weitere", "{} and {more} more", q.join(", ")) } else { q.join(", ") }
}

// ───────────────────────────── Tests ─────────────────────────────
// `cargo test` führt alle Funktionen mit `#[test]` aus.

#[cfg(test)]
mod tests {
    use super::*;

    /// Legt einen frischen Testordner mit ein paar Dateien an.
    fn sandbox(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("kompass-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Fotos")).unwrap();
        std::fs::write(dir.join("notizen.txt"), "hallo").unwrap();
        std::fs::write(dir.join("a.jpg"), "").unwrap();
        std::fs::write(dir.join("b.jpg"), "").unwrap();
        std::fs::write(dir.join("Fotos/a.jpg"), "").unwrap();
        dir
    }

    #[test]
    fn sichere_befehle() {
        let d = sandbox("safe");
        for line in ["ls -la", "cd Fotos", "pwd", "echo hallo", "cat notizen.txt", "ls | grep jpg", "echo x >> notizen.txt", "echo x > neu.txt", "ls > /dev/null", "git status"] {
            assert_eq!(check(line, &d).level, Level::Safe, "{line}");
        }
    }

    #[test]
    fn rm_wird_zum_papierkorb() {
        let d = sandbox("rm");
        let v = check("rm -rf Fotos 'notizen.txt'", &d);
        assert_eq!(v.level, Level::Danger);
        let plan = v.trash.expect("Papierkorb-Plan erwartet");
        assert_eq!(plan.rewritten, "kompass-trash -rf Fotos 'notizen.txt'");
        assert_eq!(plan.targets.len(), 2);
        assert_eq!(plan.targets[0].kind, "dir");
        assert_eq!(plan.targets[0].entries, Some(1));
    }

    #[test]
    fn rm_mit_muster_und_verknuepfung() {
        let d = sandbox("glob");
        let v = check("cd . && rm *.jpg; ls", &d);
        let plan = v.trash.unwrap();
        assert_eq!(plan.rewritten, "cd . && kompass-trash *.jpg; ls");
        assert_eq!(plan.targets.len(), 2); // a.jpg, b.jpg
        assert!(v.findings[0].text.contains("2 Dateien"));
    }

    #[test]
    fn sudo_rm_nicht_in_papierkorb() {
        let d = sandbox("sudo");
        let v = check("sudo rm -rf /Library/Foo", &d);
        assert_eq!(v.level, Level::Danger);
        assert!(v.trash.is_none());
        assert!(v.findings.iter().any(|f| f.title.contains("Administrator")));
    }

    #[test]
    fn umleitung_auf_bestehende_datei() {
        let d = sandbox("redir");
        assert_eq!(check("echo neu > notizen.txt", &d).level, Level::Danger);
        assert_eq!(check("ls 2>notizen.txt", &d).level, Level::Danger);
        assert_eq!(check("echo 'a > notizen.txt'", &d).level, Level::Safe); // nur Text in Anführungszeichen
    }

    #[test]
    fn aufpassen_bei_ueberschreiben() {
        let d = sandbox("cp");
        assert_eq!(check("cp a.jpg Fotos/", &d).level, Level::Care);
        assert_eq!(check("mv b.jpg notizen.txt", &d).level, Level::Care);
        assert_eq!(check("cp b.jpg Fotos/", &d).level, Level::Safe); // Fotos/b.jpg gibt es nicht
        assert_eq!(check("cp -n a.jpg Fotos/", &d).level, Level::Safe);
    }

    #[test]
    fn weitere_gefahren() {
        let d = sandbox("misc");
        assert_eq!(check("chmod -R 777 Fotos", &d).level, Level::Danger);
        assert_eq!(check("curl -fsSL https://example.com/x.sh | bash", &d).level, Level::Danger);
        assert_eq!(check("find . -name '*.jpg' -delete", &d).level, Level::Danger);
        assert_eq!(check("git reset --hard", &d).level, Level::Danger);
        assert_eq!(check("brew install wget", &d).level, Level::Care);
        assert_eq!(check("echo rm", &d).level, Level::Safe);
    }
}
