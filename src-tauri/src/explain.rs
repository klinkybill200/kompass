//! Live-Erklärung: übersetzt eine (noch nicht abgeschickte) Befehlszeile in Klartext.
//!
//! Beispiel: `ls -la Fotos` →
//!   Zusammenfassung: „Zeigt, was in „Fotos“ liegt – als Liste mit Details, auch versteckte Dateien.“
//!   Bausteine: [ls: zeigt Ordnerinhalt] [-la: l = Liste mit Details · a = auch versteckte] [Fotos: Ordner]

use crate::guard::{self, base, is_assignment, resolve, Level, Segment, Word};
use crate::i18n::{is_en, tr};
use serde::Serialize;
use std::path::Path;

/// Ein erklärter Baustein der Zeile (wird im Frontend als „Chip“ gezeigt).
#[derive(Debug, Clone, Serialize)]
pub struct Part {
    pub token: String,
    pub meaning: String,
    /// "cmd", "flag", "arg", "op", "redir" oder "unknown"
    pub kind: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct Explanation {
    pub summary: String,
    pub parts: Vec<Part>,
    pub level: Level,
}

/// Wie die Argumente eines Befehls zu verstehen sind.
/// `#[derive(Clone, Copy)]` erlaubt einfaches Kopieren (kleine Werte).
#[derive(Clone, Copy, PartialEq)]
enum Args {
    Paths,   // Dateien/Ordner
    Text,    // freier Text (echo, say)
    Pattern, // erstes Argument = Suchbegriff, danach Dateien (grep)
    Sub,     // erstes Argument = Unterbefehl (git, brew …)
    Program, // erstes Argument ist selbst ein Befehl (sudo, man, which)
    Other,   // irgendetwas anderes (Adresse, Zahl …)
}

/// Lexikon-Eintrag. `&'static str` = Text, der fest im Programm steht.
struct Cmd {
    name: &'static str,
    /// Beschreibung mit Argumenten; `{}` wird durch die Argumente ersetzt.
    with_args: &'static str,
    /// Beschreibung ohne Argumente.
    plain: &'static str,
    args: Args,
    flags: &'static [(&'static str, &'static str)],
    subs: &'static [(&'static str, &'static str)],
}

// `const` = feste Werte, die schon beim Kompilieren feststehen.
const NO: &[(&str, &str)] = &[];

const COMMANDS: &[Cmd] = &[
    Cmd { name: "ls", with_args: "Zeigt, was in {} liegt", plain: "Zeigt, was im aktuellen Ordner liegt", args: Args::Paths,
        flags: &[("l", "als Liste mit Details"), ("a", "auch versteckte Dateien"), ("A", "auch versteckte (ohne . und ..)"), ("h", "Größen lesbar (KB, MB)"), ("t", "neueste zuerst"), ("r", "umgekehrte Reihenfolge"), ("R", "inklusive aller Unterordner"), ("S", "größte zuerst"), ("1", "ein Eintrag pro Zeile"), ("G", "farbig")], subs: NO },
    Cmd { name: "cd", with_args: "Wechselt in den Ordner {}", plain: "Wechselt zurück in deinen Benutzerordner", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "pwd", with_args: "Zeigt, in welchem Ordner du gerade bist", plain: "Zeigt, in welchem Ordner du gerade bist", args: Args::Other, flags: NO, subs: NO },
    Cmd { name: "mkdir", with_args: "Legt den neuen Ordner {} an", plain: "Legt einen neuen Ordner an (Name fehlt noch)", args: Args::Paths,
        flags: &[("p", "samt fehlender Zwischenordner, ohne Fehler wenn er schon existiert")], subs: NO },
    Cmd { name: "touch", with_args: "Legt die leere Datei {} an (gibt es sie schon, bekommt sie nur ein neues Datum)", plain: "Legt eine leere Datei an (Name fehlt noch)", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "cat", with_args: "Zeigt den Inhalt von {}", plain: "Wartet auf Text von der Tastatur (Ctrl+D beendet)", args: Args::Paths,
        flags: &[("n", "mit Zeilennummern")], subs: NO },
    Cmd { name: "less", with_args: "Zeigt {} seitenweise an (Leertaste blättert, q beendet)", plain: "Zeigt Text seitenweise an", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "more", with_args: "Zeigt {} seitenweise an (q beendet)", plain: "Zeigt Text seitenweise an", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "head", with_args: "Zeigt den Anfang von {}", plain: "Zeigt den Anfang eines Textes", args: Args::Paths,
        flags: &[("n", "Anzahl der Zeilen")], subs: NO },
    Cmd { name: "tail", with_args: "Zeigt das Ende von {}", plain: "Zeigt das Ende eines Textes", args: Args::Paths,
        flags: &[("n", "Anzahl der Zeilen"), ("f", "bleibt dran und zeigt neue Zeilen live (Ctrl+C beendet)")], subs: NO },
    Cmd { name: "cp", with_args: "Kopiert {}", plain: "Kopiert Dateien (Quelle und Ziel fehlen noch)", args: Args::Paths,
        flags: &[("r", "ganze Ordner samt Inhalt"), ("R", "ganze Ordner samt Inhalt"), ("i", "fragt vor dem Überschreiben"), ("n", "überschreibt nie"), ("v", "zeigt jede kopierte Datei")], subs: NO },
    Cmd { name: "mv", with_args: "Verschiebt bzw. benennt um: {}", plain: "Verschiebt oder benennt um (Quelle und Ziel fehlen noch)", args: Args::Paths,
        flags: &[("i", "fragt vor dem Überschreiben"), ("n", "überschreibt nie"), ("v", "zeigt, was passiert")], subs: NO },
    Cmd { name: "rm", with_args: "Löscht {} – in Kompass landet es im Papierkorb", plain: "Löscht Dateien (welche, fehlt noch)", args: Args::Paths,
        flags: &[("r", "auch Ordner samt Inhalt"), ("R", "auch Ordner samt Inhalt"), ("f", "ohne Rückfrage, ohne Fehlermeldung"), ("i", "fragt bei jeder Datei nach"), ("v", "zeigt jede gelöschte Datei")], subs: NO },
    Cmd { name: "rmdir", with_args: "Entfernt den leeren Ordner {}", plain: "Entfernt leere Ordner", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "echo", with_args: "Gibt den Text {} aus", plain: "Gibt eine leere Zeile aus", args: Args::Text,
        flags: &[("n", "ohne Zeilenumbruch am Ende")], subs: NO },
    Cmd { name: "say", with_args: "Dein Mac spricht {} laut aus", plain: "Dein Mac spricht Text laut aus", args: Args::Text,
        flags: &[("v", "mit dieser Stimme")], subs: NO },
    Cmd { name: "open", with_args: "Öffnet {} mit dem passenden Programm", plain: "Öffnet Dateien, Ordner oder Programme", args: Args::Paths,
        flags: &[("a", "mit diesem Programm"), ("R", "zeigt es im Finder")], subs: NO },
    Cmd { name: "clear", with_args: "Räumt das Fenster auf", plain: "Räumt das Fenster auf (der Verlauf bleibt)", args: Args::Other, flags: NO, subs: NO },
    Cmd { name: "history", with_args: "Zeigt deine zuletzt getippten Befehle", plain: "Zeigt deine zuletzt getippten Befehle", args: Args::Other, flags: NO, subs: NO },
    Cmd { name: "man", with_args: "Öffnet die Anleitung zu {} (q beendet)", plain: "Öffnet eine Anleitung", args: Args::Program, flags: NO, subs: NO },
    Cmd { name: "which", with_args: "Zeigt, wo das Programm {} gespeichert ist", plain: "Zeigt, wo ein Programm gespeichert ist", args: Args::Program, flags: NO, subs: NO },
    Cmd { name: "grep", with_args: "Sucht Zeilen mit einem Suchbegriff in {}", plain: "Sucht Zeilen, die einen Suchbegriff enthalten", args: Args::Pattern,
        flags: &[("i", "Groß/Klein egal"), ("r", "in allen Unterordnern"), ("R", "in allen Unterordnern"), ("n", "mit Zeilennummern"), ("v", "nur Zeilen OHNE den Begriff"), ("c", "nur zählen"), ("l", "nur Dateinamen zeigen")], subs: NO },
    Cmd { name: "find", with_args: "Sucht Dateien in {}", plain: "Sucht Dateien", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "wc", with_args: "Zählt Zeilen, Wörter und Zeichen in {}", plain: "Zählt Zeilen, Wörter und Zeichen", args: Args::Paths,
        flags: &[("l", "nur Zeilen"), ("w", "nur Wörter"), ("c", "nur Bytes")], subs: NO },
    Cmd { name: "sort", with_args: "Sortiert die Zeilen von {}", plain: "Sortiert Zeilen", args: Args::Paths,
        flags: &[("r", "rückwärts"), ("n", "nach Zahlen"), ("u", "doppelte weglassen")], subs: NO },
    Cmd { name: "uniq", with_args: "Entfernt doppelte Zeilen in {}", plain: "Entfernt direkt aufeinanderfolgende doppelte Zeilen", args: Args::Paths,
        flags: &[("c", "mit Anzahl")], subs: NO },
    Cmd { name: "whoami", with_args: "Zeigt deinen Benutzernamen", plain: "Zeigt deinen Benutzernamen", args: Args::Other, flags: NO, subs: NO },
    Cmd { name: "date", with_args: "Zeigt Datum und Uhrzeit", plain: "Zeigt Datum und Uhrzeit", args: Args::Other, flags: NO, subs: NO },
    Cmd { name: "cal", with_args: "Zeigt einen Kalender", plain: "Zeigt den Kalender dieses Monats", args: Args::Other, flags: NO, subs: NO },
    Cmd { name: "du", with_args: "Zeigt, wie viel Platz {} belegt", plain: "Zeigt, wie viel Platz Ordner belegen", args: Args::Paths,
        flags: &[("h", "Größen lesbar (KB, MB, GB)"), ("s", "nur die Summe"), ("d", "bis zu dieser Ordnertiefe")], subs: NO },
    Cmd { name: "df", with_args: "Zeigt freien Speicherplatz", plain: "Zeigt freien Speicherplatz auf den Laufwerken", args: Args::Other,
        flags: &[("h", "Größen lesbar (GB)")], subs: NO },
    Cmd { name: "top", with_args: "Zeigt live, welche Programme gerade arbeiten (q beendet)", plain: "Zeigt live, welche Programme gerade arbeiten (q beendet)", args: Args::Other, flags: NO, subs: NO },
    Cmd { name: "ps", with_args: "Listet laufende Programme auf", plain: "Listet laufende Programme auf", args: Args::Other, flags: NO, subs: NO },
    Cmd { name: "kill", with_args: "Beendet das Programm mit der Nummer {}", plain: "Beendet ein Programm (Nummer fehlt noch)", args: Args::Other, flags: NO, subs: NO },
    Cmd { name: "killall", with_args: "Beendet alle Programme namens {}", plain: "Beendet Programme nach Namen", args: Args::Other, flags: NO, subs: NO },
    Cmd { name: "chmod", with_args: "Ändert die Zugriffsrechte: {}", plain: "Ändert Zugriffsrechte", args: Args::Other,
        flags: &[("R", "auch für alles in den Ordnern")], subs: NO },
    Cmd { name: "chown", with_args: "Ändert den Besitzer: {}", plain: "Ändert den Besitzer", args: Args::Other,
        flags: &[("R", "auch für alles in den Ordnern")], subs: NO },
    Cmd { name: "sudo", with_args: "Führt {} mit Administrator-Rechten aus", plain: "Führt einen Befehl mit Administrator-Rechten aus", args: Args::Program, flags: NO, subs: NO },
    Cmd { name: "nano", with_args: "Öffnet {} im einfachen Texteditor (Ctrl+X beendet)", plain: "Öffnet den einfachen Texteditor (Ctrl+X beendet)", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "vim", with_args: "Öffnet {} im Editor vim (Raus: Esc, dann :q! und Enter)", plain: "Öffnet den Editor vim (Raus: Esc, dann :q! und Enter)", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "vi", with_args: "Öffnet {} im Editor vi (Raus: Esc, dann :q! und Enter)", plain: "Öffnet den Editor vi (Raus: Esc, dann :q! und Enter)", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "code", with_args: "Öffnet {} in Visual Studio Code", plain: "Öffnet Visual Studio Code", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "tree", with_args: "Zeigt {} als Baum mit allen Unterordnern", plain: "Zeigt den Ordner als Baum", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "file", with_args: "Verrät, was für eine Art Datei {} ist", plain: "Verrät, was für eine Art Datei etwas ist", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "diff", with_args: "Vergleicht {} und zeigt die Unterschiede", plain: "Vergleicht zwei Dateien", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "ln", with_args: "Legt eine Verknüpfung an: {}", plain: "Legt eine Verknüpfung an", args: Args::Paths,
        flags: &[("s", "als „Alias“ (symbolische Verknüpfung)")], subs: NO },
    Cmd { name: "zip", with_args: "Packt {} in ein ZIP-Archiv", plain: "Packt Dateien in ein ZIP-Archiv", args: Args::Paths,
        flags: &[("r", "ganze Ordner")], subs: NO },
    Cmd { name: "unzip", with_args: "Entpackt das ZIP-Archiv {}", plain: "Entpackt ein ZIP-Archiv", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "tar", with_args: "Packt oder entpackt ein Archiv: {}", plain: "Packt oder entpackt Archive", args: Args::Paths,
        flags: &[("x", "entpacken"), ("c", "packen"), ("z", "mit gzip-Kompression"), ("v", "zeigt jede Datei"), ("f", "Archiv-Datei")], subs: NO },
    Cmd { name: "curl", with_args: "Holt {} aus dem Internet", plain: "Holt etwas aus dem Internet", args: Args::Other,
        flags: &[("O", "speichert es als Datei"), ("o", "speichert es unter diesem Namen"), ("L", "folgt Weiterleitungen"), ("s", "ohne Fortschrittsanzeige"), ("I", "nur die Kopfzeilen")], subs: NO },
    Cmd { name: "ping", with_args: "Prüft, ob {} erreichbar ist (Ctrl+C beendet)", plain: "Prüft, ob ein Rechner erreichbar ist", args: Args::Other,
        flags: &[("c", "nur so oft")], subs: NO },
    Cmd { name: "ssh", with_args: "Verbindet dich mit dem entfernten Rechner {}", plain: "Verbindet dich mit einem entfernten Rechner", args: Args::Other, flags: NO, subs: NO },
    Cmd { name: "exit", with_args: "Beendet diese Terminal-Sitzung", plain: "Beendet diese Terminal-Sitzung", args: Args::Other, flags: NO, subs: NO },
    Cmd { name: "python3", with_args: "Startet das Python-Programm {}", plain: "Startet Python zum Ausprobieren (exit() beendet)", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "node", with_args: "Startet das JavaScript-Programm {}", plain: "Startet Node.js zum Ausprobieren (Ctrl+D beendet)", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "caffeinate", with_args: "Hält den Mac wach", plain: "Hält den Mac wach, bis du Ctrl+C drückst", args: Args::Other, flags: NO, subs: NO },
    Cmd { name: "kompass-trash", with_args: "Legt {} in den Papierkorb", plain: "Legt Dateien in den Papierkorb", args: Args::Paths, flags: NO, subs: NO },
    Cmd { name: "kompass-undo", with_args: "Holt die zuletzt gelöschten Dateien zurück", plain: "Holt die zuletzt gelöschten Dateien zurück", args: Args::Other, flags: NO, subs: NO },
    Cmd { name: "rueckgaengig", with_args: "Holt die zuletzt gelöschten Dateien zurück", plain: "Holt die zuletzt gelöschten Dateien zurück", args: Args::Other, flags: NO, subs: NO },
    Cmd { name: "git", with_args: "Versionsverwaltung: {}", plain: "Versionsverwaltung git (Unterbefehl fehlt noch)", args: Args::Sub, flags: NO,
        subs: &[("status", "zeigt, was sich geändert hat"), ("log", "zeigt die bisherigen Speicherstände"), ("diff", "zeigt die Änderungen im Detail"), ("add", "merkt Änderungen zum Speichern vor"), ("commit", "speichert vorgemerkte Änderungen als neuen Stand"), ("push", "lädt deine Stände hoch"), ("pull", "holt neue Stände herunter"), ("clone", "lädt ein Projekt herunter"), ("init", "macht diesen Ordner zu einem git-Projekt"), ("branch", "zeigt oder verwaltet Zweige"), ("checkout", "wechselt den Zweig oder stellt Dateien wieder her"), ("switch", "wechselt den Zweig"), ("reset", "setzt Änderungen zurück"), ("stash", "legt Änderungen beiseite")] },
    Cmd { name: "brew", with_args: "Paketmanager Homebrew: {}", plain: "Paketmanager Homebrew (Unterbefehl fehlt noch)", args: Args::Sub, flags: NO,
        subs: &[("install", "installiert ein Programm"), ("uninstall", "entfernt ein Programm"), ("update", "holt die neueste Programmliste"), ("upgrade", "aktualisiert installierte Programme"), ("list", "zeigt installierte Programme"), ("search", "sucht nach Programmen"), ("info", "zeigt Infos zu einem Programm")] },
    Cmd { name: "npm", with_args: "JavaScript-Pakete: {}", plain: "JavaScript-Paketmanager (Unterbefehl fehlt noch)", args: Args::Sub, flags: NO,
        subs: &[("install", "installiert Pakete"), ("i", "installiert Pakete"), ("run", "startet ein Skript aus package.json"), ("start", "startet das Projekt"), ("test", "startet die Tests")] },
];


/// Englische Texte zum Lexikon (gleiche Reihenfolge der Felder wie `Cmd`).
/// Fehlt ein Befehl hier, wird der deutsche Text verwendet.
struct CmdEn {
    name: &'static str,
    with_args: &'static str,
    plain: &'static str,
    flags: &'static [(&'static str, &'static str)],
    subs: &'static [(&'static str, &'static str)],
}

const EN: &[CmdEn] = &[
    CmdEn { name: "ls", with_args: "Shows what is in {}", plain: "Shows what is in the current folder",
        flags: &[("l", "as a detailed list"), ("a", "including hidden files"), ("A", "including hidden (without . and ..)"), ("h", "readable sizes (KB, MB)"), ("t", "newest first"), ("r", "reverse order"), ("R", "including all subfolders"), ("S", "largest first"), ("1", "one entry per line"), ("G", "in color")], subs: NO },
    CmdEn { name: "cd", with_args: "Changes into the folder {}", plain: "Goes back to your home folder", flags: NO, subs: NO },
    CmdEn { name: "pwd", with_args: "Shows which folder you are in", plain: "Shows which folder you are in", flags: NO, subs: NO },
    CmdEn { name: "mkdir", with_args: "Creates the new folder {}", plain: "Creates a new folder (name still missing)", flags: &[("p", "with missing parent folders, no error if it exists")], subs: NO },
    CmdEn { name: "touch", with_args: "Creates the empty file {} (if it exists, it only gets a new date)", plain: "Creates an empty file (name still missing)", flags: NO, subs: NO },
    CmdEn { name: "cat", with_args: "Shows the content of {}", plain: "Waits for text from the keyboard (Ctrl+D ends)", flags: &[("n", "with line numbers")], subs: NO },
    CmdEn { name: "less", with_args: "Shows {} page by page (space scrolls, q quits)", plain: "Shows text page by page", flags: NO, subs: NO },
    CmdEn { name: "more", with_args: "Shows {} page by page (q quits)", plain: "Shows text page by page", flags: NO, subs: NO },
    CmdEn { name: "head", with_args: "Shows the beginning of {}", plain: "Shows the beginning of a text", flags: &[("n", "number of lines")], subs: NO },
    CmdEn { name: "tail", with_args: "Shows the end of {}", plain: "Shows the end of a text", flags: &[("n", "number of lines"), ("f", "keeps watching and shows new lines live (Ctrl+C ends)")], subs: NO },
    CmdEn { name: "cp", with_args: "Copies {}", plain: "Copies files (source and target still missing)", flags: &[("r", "whole folders with content"), ("R", "whole folders with content"), ("i", "asks before overwriting"), ("n", "never overwrites"), ("v", "shows every copied file")], subs: NO },
    CmdEn { name: "mv", with_args: "Moves or renames: {}", plain: "Moves or renames (source and target still missing)", flags: &[("i", "asks before overwriting"), ("n", "never overwrites"), ("v", "shows what happens")], subs: NO },
    CmdEn { name: "rm", with_args: "Deletes {} – in Kompass it goes to the Trash", plain: "Deletes files (which ones is still missing)", flags: &[("r", "also folders with content"), ("R", "also folders with content"), ("f", "without asking, without error messages"), ("i", "asks for every file"), ("v", "shows every deleted file")], subs: NO },
    CmdEn { name: "rmdir", with_args: "Removes the empty folder {}", plain: "Removes empty folders", flags: NO, subs: NO },
    CmdEn { name: "echo", with_args: "Prints the text {}", plain: "Prints an empty line", flags: &[("n", "without a line break at the end")], subs: NO },
    CmdEn { name: "say", with_args: "Your Mac speaks {} out loud", plain: "Your Mac speaks text out loud", flags: &[("v", "with this voice")], subs: NO },
    CmdEn { name: "open", with_args: "Opens {} with the matching app", plain: "Opens files, folders or apps", flags: &[("a", "with this app"), ("R", "reveals it in Finder")], subs: NO },
    CmdEn { name: "clear", with_args: "Tidies up the window", plain: "Tidies up the window (history stays)", flags: NO, subs: NO },
    CmdEn { name: "history", with_args: "Shows the commands you typed recently", plain: "Shows the commands you typed recently", flags: NO, subs: NO },
    CmdEn { name: "man", with_args: "Opens the manual for {} (q quits)", plain: "Opens a manual", flags: NO, subs: NO },
    CmdEn { name: "which", with_args: "Shows where the program {} is stored", plain: "Shows where a program is stored", flags: NO, subs: NO },
    CmdEn { name: "grep", with_args: "Searches lines containing a search term in {}", plain: "Searches lines that contain a search term", flags: &[("i", "ignore case"), ("r", "in all subfolders"), ("R", "in all subfolders"), ("n", "with line numbers"), ("v", "only lines WITHOUT the term"), ("c", "only count"), ("l", "only show file names")], subs: NO },
    CmdEn { name: "find", with_args: "Searches files in {}", plain: "Searches files", flags: NO, subs: NO },
    CmdEn { name: "wc", with_args: "Counts lines, words and characters in {}", plain: "Counts lines, words and characters", flags: &[("l", "only lines"), ("w", "only words"), ("c", "only bytes")], subs: NO },
    CmdEn { name: "sort", with_args: "Sorts the lines of {}", plain: "Sorts lines", flags: &[("r", "reversed"), ("n", "numerically"), ("u", "drop duplicates")], subs: NO },
    CmdEn { name: "uniq", with_args: "Removes duplicate lines in {}", plain: "Removes directly repeated lines", flags: &[("c", "with count")], subs: NO },
    CmdEn { name: "whoami", with_args: "Shows your user name", plain: "Shows your user name", flags: NO, subs: NO },
    CmdEn { name: "date", with_args: "Shows date and time", plain: "Shows date and time", flags: NO, subs: NO },
    CmdEn { name: "cal", with_args: "Shows a calendar", plain: "Shows this month's calendar", flags: NO, subs: NO },
    CmdEn { name: "du", with_args: "Shows how much space {} uses", plain: "Shows how much space folders use", flags: &[("h", "readable sizes (KB, MB, GB)"), ("s", "only the total"), ("d", "down to this folder depth")], subs: NO },
    CmdEn { name: "df", with_args: "Shows free disk space", plain: "Shows free space on your drives", flags: &[("h", "readable sizes (GB)")], subs: NO },
    CmdEn { name: "top", with_args: "Shows live which programs are working (q quits)", plain: "Shows live which programs are working (q quits)", flags: NO, subs: NO },
    CmdEn { name: "ps", with_args: "Lists running programs", plain: "Lists running programs", flags: NO, subs: NO },
    CmdEn { name: "kill", with_args: "Quits the program with number {}", plain: "Quits a program (number still missing)", flags: NO, subs: NO },
    CmdEn { name: "killall", with_args: "Quits all programs named {}", plain: "Quits programs by name", flags: NO, subs: NO },
    CmdEn { name: "chmod", with_args: "Changes permissions: {}", plain: "Changes permissions", flags: &[("R", "also for everything inside the folders")], subs: NO },
    CmdEn { name: "chown", with_args: "Changes the owner: {}", plain: "Changes the owner", flags: &[("R", "also for everything inside the folders")], subs: NO },
    CmdEn { name: "sudo", with_args: "Runs {} with administrator rights", plain: "Runs a command with administrator rights", flags: NO, subs: NO },
    CmdEn { name: "nano", with_args: "Opens {} in the simple text editor (Ctrl+X quits)", plain: "Opens the simple text editor (Ctrl+X quits)", flags: NO, subs: NO },
    CmdEn { name: "vim", with_args: "Opens {} in the vim editor (to exit: Esc, then :q! and Enter)", plain: "Opens the vim editor (to exit: Esc, then :q! and Enter)", flags: NO, subs: NO },
    CmdEn { name: "vi", with_args: "Opens {} in the vi editor (to exit: Esc, then :q! and Enter)", plain: "Opens the vi editor (to exit: Esc, then :q! and Enter)", flags: NO, subs: NO },
    CmdEn { name: "code", with_args: "Opens {} in Visual Studio Code", plain: "Opens Visual Studio Code", flags: NO, subs: NO },
    CmdEn { name: "tree", with_args: "Shows {} as a tree with all subfolders", plain: "Shows the folder as a tree", flags: NO, subs: NO },
    CmdEn { name: "file", with_args: "Tells what kind of file {} is", plain: "Tells what kind of file something is", flags: NO, subs: NO },
    CmdEn { name: "diff", with_args: "Compares {} and shows the differences", plain: "Compares two files", flags: NO, subs: NO },
    CmdEn { name: "ln", with_args: "Creates a link: {}", plain: "Creates a link", flags: &[("s", "as an “alias” (symbolic link)")], subs: NO },
    CmdEn { name: "zip", with_args: "Packs {} into a ZIP archive", plain: "Packs files into a ZIP archive", flags: &[("r", "whole folders")], subs: NO },
    CmdEn { name: "unzip", with_args: "Unpacks the ZIP archive {}", plain: "Unpacks a ZIP archive", flags: NO, subs: NO },
    CmdEn { name: "tar", with_args: "Packs or unpacks an archive: {}", plain: "Packs or unpacks archives", flags: &[("x", "unpack"), ("c", "pack"), ("z", "with gzip compression"), ("v", "shows every file"), ("f", "archive file")], subs: NO },
    CmdEn { name: "curl", with_args: "Fetches {} from the internet", plain: "Fetches something from the internet", flags: &[("O", "saves it as a file"), ("o", "saves it under this name"), ("L", "follows redirects"), ("s", "without progress display"), ("I", "headers only")], subs: NO },
    CmdEn { name: "ping", with_args: "Checks whether {} is reachable (Ctrl+C ends)", plain: "Checks whether a computer is reachable", flags: &[("c", "only this many times")], subs: NO },
    CmdEn { name: "ssh", with_args: "Connects you to the remote computer {}", plain: "Connects you to a remote computer", flags: NO, subs: NO },
    CmdEn { name: "exit", with_args: "Ends this terminal session", plain: "Ends this terminal session", flags: NO, subs: NO },
    CmdEn { name: "python3", with_args: "Runs the Python program {}", plain: "Starts Python to experiment (exit() ends)", flags: NO, subs: NO },
    CmdEn { name: "node", with_args: "Runs the JavaScript program {}", plain: "Starts Node.js to experiment (Ctrl+D ends)", flags: NO, subs: NO },
    CmdEn { name: "caffeinate", with_args: "Keeps the Mac awake", plain: "Keeps the Mac awake until you press Ctrl+C", flags: NO, subs: NO },
    CmdEn { name: "kompass-trash", with_args: "Moves {} to the Trash", plain: "Moves files to the Trash", flags: NO, subs: NO },
    CmdEn { name: "kompass-undo", with_args: "Brings back the most recently deleted files", plain: "Brings back the most recently deleted files", flags: NO, subs: NO },
    CmdEn { name: "rueckgaengig", with_args: "Brings back the most recently deleted files", plain: "Brings back the most recently deleted files", flags: NO, subs: NO },
    CmdEn { name: "git", with_args: "Version control: {}", plain: "Version control git (subcommand still missing)", flags: NO,
        subs: &[("status", "shows what has changed"), ("log", "shows the saved versions"), ("diff", "shows the changes in detail"), ("add", "marks changes for saving"), ("commit", "saves marked changes as a new version"), ("push", "uploads your versions"), ("pull", "downloads new versions"), ("clone", "downloads a project"), ("init", "turns this folder into a git project"), ("branch", "shows or manages branches"), ("checkout", "switches branch or restores files"), ("switch", "switches branch"), ("reset", "resets changes"), ("stash", "puts changes aside")] },
    CmdEn { name: "brew", with_args: "Homebrew package manager: {}", plain: "Homebrew package manager (subcommand still missing)", flags: NO,
        subs: &[("install", "installs a program"), ("uninstall", "removes a program"), ("update", "fetches the latest program list"), ("upgrade", "updates installed programs"), ("list", "shows installed programs"), ("search", "searches programs"), ("info", "shows info about a program")] },
    CmdEn { name: "npm", with_args: "JavaScript packages: {}", plain: "JavaScript package manager (subcommand still missing)", flags: NO,
        subs: &[("install", "installs packages"), ("i", "installs packages"), ("run", "runs a script from package.json"), ("start", "starts the project"), ("test", "runs the tests")] },
];

fn en(c: &Cmd) -> Option<&'static CmdEn> {
    if is_en() { EN.iter().find(|e| e.name == c.name) } else { None }
}
fn with_args_text(c: &Cmd) -> &'static str {
    en(c).map(|e| e.with_args).unwrap_or(c.with_args)
}
fn plain_text(c: &Cmd) -> &'static str {
    en(c).map(|e| e.plain).unwrap_or(c.plain)
}
fn flag_list(c: &Cmd) -> &'static [(&'static str, &'static str)] {
    en(c).map(|e| e.flags).unwrap_or(c.flags)
}
fn sub_list(c: &Cmd) -> &'static [(&'static str, &'static str)] {
    en(c).map(|e| e.subs).unwrap_or(c.subs)
}

/// Text in typografischen Anführungszeichen der jeweiligen Sprache.
pub fn quote(s: &str) -> String {
    if is_en() { format!("“{s}”") } else { format!("„{s}“") }
}

/// Sucht einen Befehl im Lexikon. `Option<&Cmd>` = vielleicht gefunden.
fn lookup(name: &str) -> Option<&'static Cmd> {
    COMMANDS.iter().find(|c| c.name == name)
}

/// Alle Befehlsnamen, die Kompass kennt (für Tippfehler-Vorschläge).
pub fn known_names() -> impl Iterator<Item = &'static str> {
    COMMANDS.iter().map(|c| c.name)
}

/// „Meintest du …?“ – findet den ähnlichsten bekannten Befehl.
pub fn suggest_command(name: &str) -> Option<&'static str> {
    // Häufige Verwechslungen auf dem Mac
    match name {
        "python" => return Some("python3"),
        "pip" => return Some("pip3"),
        "dir" => return Some("ls"),
        "cls" => return Some("clear"),
        "copy" => return Some("cp"),
        "move" => return Some("mv"),
        "del" | "delete" => return Some("rm"),
        "md" => return Some("mkdir"),
        "type" => return Some("cat"),
        _ => {}
    }
    let lower = name.to_lowercase();
    // Nur falsche Großschreibung (`LS`)? Dann ist es genau dieser Befehl.
    if lower != name {
        if let Some(c) = lookup(&lower) {
            return Some(c.name);
        }
    }
    known_names()
        .filter(|k| !k.starts_with("kompass") && *k != "rueckgaengig")
        .map(|k| (k, distance(&lower, k)))
        .filter(|(k, d)| *d > 0 && *d <= if k.len() <= 3 { 1 } else { 2 })
        .min_by_key(|(_, d)| *d)
        .map(|(k, _)| k)
}

/// Levenshtein-Abstand: wie viele Tipp-Änderungen machen aus a das Wort b?
/// Vertauschte Nachbarbuchstaben (`sl` → `ls`) zählen als ein einziger Fehler.
pub fn distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    // `vec![vec![0; n]; m]` = Tabelle mit m Zeilen und n Spalten, gefüllt mit 0
    let mut d = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for i in 0..=a.len() { d[i][0] = i; }
    for j in 0..=b.len() { d[0][j] = j; }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            d[i][j] = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[a.len()][b.len()]
}

/// Hauptfunktion: erklärt eine Zeile. `cwd` = aktueller Ordner (für „gibt es diese Datei?“).
pub fn explain(line: &str, cwd: &Path) -> Explanation {
    let level = guard::check(line, cwd).level;
    let segments = guard::split_segments(guard::tokenize(line));
    let mut parts = Vec::new();
    let mut summaries = Vec::new();

    for seg in &segments {
        if let Some(op) = &seg.op_before {
            if let Some(meaning) = op_meaning(op) {
                parts.push(Part { token: op.clone(), meaning: meaning.into(), kind: "op" });
            }
        }
        if let Some(s) = explain_segment(seg, cwd, &mut parts) {
            summaries.push(s);
        }
        for (op, target) in &seg.redirs {
            let name = target.as_ref().map(|t| t.text.as_str()).unwrap_or("…");
            parts.push(Part { token: format!("{op} {name}"), meaning: redir_meaning(op, name, cwd), kind: "redir" });
        }
    }

    let summary = match summaries.len() {
        0 => String::new(),
        1 => format!("{}.", summaries[0]),
        _ => format!("{}.", summaries.join(&tr(" – dann: ", " – then: "))),
    };
    Explanation { summary, parts, level }
}

fn op_meaning(op: &str) -> Option<&'static str> {
    Some(match op {
        "&&" if is_en() => "then – but only if the previous one worked",
        "||" if is_en() => "only if the previous one did NOT work",
        ";" if is_en() => "afterwards (whether or not the previous one worked)",
        "|" if is_en() => "passes the output on to the next command",
        "&" if is_en() => "keeps running in the background",
        "&&" => "dann – aber nur, wenn das vorige geklappt hat",
        "||" => "nur falls das vorige NICHT geklappt hat",
        ";" => "danach (egal, ob das vorige geklappt hat)",
        "|" => "reicht die Ausgabe weiter an den nächsten Befehl",
        "&" => "läuft im Hintergrund weiter",
        _ => return None,
    })
}

fn redir_meaning(op: &str, name: &str, cwd: &Path) -> String {
    let exists = resolve(name, cwd).is_file();
    match op.trim_start_matches(|c: char| c.is_ascii_digit()) {
        ">" | ">|" | "&>" if name.starts_with("/dev/null") => tr("wirft die Ausgabe weg", "throws the output away"),
        ">" | ">|" | "&>" if exists => t!("schreibt die Ausgabe in „{name}“ – ERSETZT den bisherigen Inhalt", "writes the output to “{name}” – REPLACES the previous content"),
        ">" | ">|" | "&>" => t!("schreibt die Ausgabe in die neue Datei „{name}“", "writes the output to the new file “{name}”"),
        ">>" | "&>>" => t!("hängt die Ausgabe hinten an „{name}“ an", "appends the output to “{name}”"),
        "<" => t!("liest die Eingabe aus „{name}“", "reads the input from “{name}”"),
        _ => tr("Umleitung", "redirection"),
    }
}

/// Erklärt einen Einzelbefehl; gibt die Zusammenfassung zurück.
fn explain_segment(seg: &Segment, cwd: &Path, parts: &mut Vec<Part>) -> Option<String> {
    let words = &seg.words;
    let mut i = 0;
    while i < words.len() && is_assignment(&words[i]) {
        parts.push(Part { token: words[i].text.clone(), meaning: tr("setzt eine Einstellung nur für diesen Befehl", "sets a setting for this command only"), kind: "arg" });
        i += 1;
    }
    let cmd_word = words.get(i)?; // `?` bei Option: kein Befehl → nichts zu erklären
    let name = base(&cmd_word.text);
    let args = &words[i + 1..];

    let Some(cmd) = lookup(name) else {
        let mut meaning = tr("Kompass kennt diesen Befehl noch nicht – vielleicht ein installiertes Programm.", "Kompass does not know this command yet – maybe an installed program.");
        if let Some(s) = suggest_command(name) {
            meaning = t!("Unbekannt. Meintest du „{s}“?", "Unknown. Did you mean “{s}”?");
        }
        parts.push(Part { token: cmd_word.text.clone(), meaning, kind: "unknown" });
        return Some(t!("Startet „{}“", "Runs “{}”", cmd_word.text));
    };

    parts.push(Part { token: cmd_word.text.clone(), meaning: plain_text(cmd).to_string(), kind: "cmd" });

    // Bei sudo/man/which ist das erste Argument selbst ein Befehl → rekursiv erklären.
    if cmd.args == Args::Program && cmd.name == "sudo" && !args.is_empty() {
        let inner = Segment { words: args.to_vec(), ..Default::default() };
        let inner_summary = explain_segment(&inner, cwd, parts)?;
        return Some(t!("Als Administrator: {}", "As administrator: {}", lowercase_first(&inner_summary)));
    }

    let mut flag_meanings: Vec<String> = Vec::new();
    let mut shown: Vec<String> = Vec::new(); // Argumente für die Zusammenfassung
    let mut opts_done = false;
    let mut first_free = true; // erstes Nicht-Option-Argument (Suchbegriff/Unterbefehl)

    for w in args {
        let t = &w.text;
        if !opts_done && t == "--" {
            opts_done = true;
            continue;
        }
        if !opts_done && t.starts_with('-') && t.len() > 1 && !(cmd.name == "cd" && t == "-") {
            let m = flag_meaning(cmd, t);
            if !m.is_empty() {
                flag_meanings.push(m.clone());
            }
            parts.push(Part {
                token: t.clone(),
                meaning: if m.is_empty() { t!("eine Option (Details: man {})", "an option (details: man {})", cmd.name) } else { m },
                kind: "flag",
            });
            continue;
        }
        let (meaning, label) = match cmd.args {
            Args::Sub if first_free => {
                let m = sub_list(cmd).iter().find(|(s, _)| s == t).map(|(_, m)| m.to_string()).unwrap_or_else(|| tr("Unterbefehl", "subcommand"));
                (m.clone(), m)
            }
            Args::Pattern if first_free => (tr("Suchbegriff", "search term"), quote(t)),
            Args::Text => (tr("Text", "text"), quote(t)),
            Args::Program => (lookup(base(t)).map(|c| plain_text(c).to_string()).unwrap_or_else(|| tr("ein Befehl", "a command")), quote(t)),
            Args::Paths | Args::Pattern => (describe_path(cmd.name, t, w, cwd), quote(t)),
            _ => (tr("Angabe", "value"), quote(t)),
        };
        first_free = false;
        parts.push(Part { token: t.clone(), meaning, kind: "arg" });
        shown.push(label);
    }

    // Zusammenfassung bauen
    let mut summary = if cmd.name == "cd" {
        match args.iter().find(|w| !w.text.starts_with('-') || w.text == "-").map(|w| w.text.as_str()) {
            None | Some("~") => tr("Wechselt zurück in deinen Benutzerordner", "Goes back to your home folder"),
            Some("..") => tr("Geht eine Ordner-Ebene nach oben", "Goes up one folder level"),
            Some("-") => tr("Springt zurück in den vorherigen Ordner", "Jumps back to the previous folder"),
            Some(t) => t!("Wechselt in den Ordner „{t}“", "Changes into the folder “{t}”"),
        }
    } else if cmd.args == Args::Sub && !shown.is_empty() {
        format!("{}: {}", cmd.name, shown[0])
    } else if shown.is_empty() {
        plain_text(cmd).to_string()
    } else {
        with_args_text(cmd).replace("{}", &join_names(&shown))
    };
    if !flag_meanings.is_empty() {
        summary.push_str(" – ");
        summary.push_str(&flag_meanings.join(", "));
    }
    Some(summary)
}

/// `-la` → „als Liste mit Details, auch versteckte Dateien“
fn flag_meaning(cmd: &Cmd, flag: &str) -> String {
    if let Some(long) = flag.strip_prefix("--") {
        return match long {
            "help" => tr("zeigt eine Kurzhilfe", "shows a short help"),
            "version" => tr("zeigt die Version", "shows the version"),
            "force" => tr("erzwingt es (ohne Rückfrage)", "forces it (without asking)"),
            "recursive" => tr("inklusive aller Unterordner", "including all subfolders"),
            _ => String::new(),
        };
    }
    let mut found = Vec::new();
    for ch in flag[1..].chars() {
        if let Some((_, m)) = flag_list(cmd).iter().find(|(f, _)| f.chars().next() == Some(ch)) {
            found.push(m.to_string());
        }
    }
    found.join(", ")
}

/// Beschreibt ein Pfad-Argument: Datei, Ordner oder „gibt es (noch) nicht“.
fn describe_path(cmd: &str, t: &str, w: &Word, cwd: &Path) -> String {
    if w.dynamic {
        return tr("wird erst beim Ausführen eingesetzt", "filled in only when it runs");
    }
    if w.glob {
        let pattern = format!("{}/{}", glob::Pattern::escape(&cwd.to_string_lossy()), t);
        let n = glob::glob(&pattern).map(|p| p.flatten().take(1000).count()).unwrap_or(0);
        return match n {
            0 => tr("Muster – passt hier auf nichts", "pattern – matches nothing here"),
            1 => tr("Muster – passt auf 1 Eintrag", "pattern – matches 1 entry"),
            n => t!("Muster – passt auf {n} Einträge", "pattern – matches {n} entries"),
        };
    }
    match t {
        "." => return tr("der aktuelle Ordner", "the current folder"),
        ".." => return tr("der Ordner eine Ebene höher", "the folder one level up"),
        "~" => return tr("dein Benutzerordner", "your home folder"),
        _ => {}
    }
    let p = resolve(t, cwd);
    match std::fs::symlink_metadata(&p) {
        Ok(m) if m.is_dir() => tr("Ordner", "folder"),
        Ok(_) => tr("Datei", "file"),
        // Bei mkdir/touch/cp/mv ist ein neuer Name ganz normal.
        Err(_) if matches!(cmd, "mkdir" | "touch" | "cp" | "mv" | "zip" | "ln" | "nano" | "vim" | "vi" | "code") => tr("neuer Name", "new name"),
        Err(_) => tr("gibt es hier nicht – Tippfehler?", "does not exist here – typo?"),
    }
}

fn join_names(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        _ => t!("{} und {}", "{} and {}", items[..items.len() - 1].join(", "), items[items.len() - 1]),
    }
}

fn lowercase_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Jeder Test bekommt einen eigenen Ordner – Tests laufen parallel.
    fn sandbox(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("kompass-explain-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("Fotos")).unwrap();
        std::fs::write(d.join("notiz.txt"), "x").unwrap();
        d
    }

    #[test]
    fn ls_mit_optionen() {
        let e = explain("ls -la Fotos", &sandbox("ls_mit_optionen"));
        assert_eq!(e.summary, "Zeigt, was in „Fotos“ liegt – als Liste mit Details, auch versteckte Dateien.");
        assert_eq!(e.parts.len(), 3);
        assert_eq!(e.parts[2].meaning, "Ordner");
    }

    #[test]
    fn cd_varianten_und_verknuepfung() {
        let d = sandbox("cd_varianten_und_verknuepfung");
        assert_eq!(explain("cd", &d).summary, "Wechselt zurück in deinen Benutzerordner.");
        assert_eq!(explain("cd ..", &d).summary, "Geht eine Ordner-Ebene nach oben.");
        let e = explain("cd Fotos && ls", &d);
        assert_eq!(e.summary, "Wechselt in den Ordner „Fotos“ – dann: Zeigt, was im aktuellen Ordner liegt.");
        assert!(e.parts.iter().any(|p| p.kind == "op" && p.token == "&&"));
    }

    #[test]
    fn tippfehler_und_umleitung() {
        let d = sandbox("tippfehler_und_umleitung");
        assert_eq!(suggest_command("sl"), Some("ls"));
        assert_eq!(suggest_command("grpe"), Some("grep"));
        assert_eq!(suggest_command("python"), Some("python3"));
        assert_eq!(suggest_command("LS"), Some("ls"));
        assert_eq!(suggest_command("zzzzzz"), None);
        let e = explain("echo hallo > notiz.txt", &d);
        assert!(e.parts.iter().any(|p| p.kind == "redir" && p.meaning.contains("ERSETZT")));
        assert_eq!(e.level, Level::Danger);
        let e = explain("cat nottiz.txt", &d);
        assert!(e.parts[1].meaning.contains("Tippfehler"));
    }

    #[test]
    fn sudo_und_git() {
        let d = sandbox("sudo_und_git");
        assert_eq!(explain("sudo mkdir neu", &d).summary, "Als Administrator: legt den neuen Ordner „neu“ an.");
        assert_eq!(explain("git status", &d).summary, "git: zeigt, was sich geändert hat.");
    }
}
