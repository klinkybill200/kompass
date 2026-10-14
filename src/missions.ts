/**
 * Lern-Missionen: kleine geführte Aufgaben.
 * Jeder Schritt prüft am echten Ergebnis eines Befehls, ob er geschafft ist.
 * Missionen sind frei wählbar; innerhalb einer Mission geht es der Reihe nach.
 * Texte sind zweisprachig: [Deutsch, Englisch].
 */
import type { L } from "./i18n";

export type MissionEvent =
  | { type: "done"; command: string; exitCode: number; cwd: string }
  | { type: "undo" }
  | { type: "translated"; query: string; command: string }
  | { type: "tab"; action: "new" | "switch" | "close" };

export type Ctx = { home: string };

export type Step = {
  id: string;
  title: L;
  text: L;
  /** Befehl, den „Einsetzen“ in die Eingabezeile schreibt (bei Klarsprache je Sprache). */
  hint?: string | L;
  /** Tastenkürzel statt Befehl (wird nur angezeigt). */
  keys?: string;
  check: (e: MissionEvent, ctx: Ctx) => boolean;
};

export type Level = "Einstieg" | "Fortgeschritten" | "Profi";
export type Mission = { id: string; emoji: string; title: L; level: Level; intro: L; steps: Step[] };

// ── kleine Helfer zum Prüfen ──
const words = (cmd: string) => cmd.trim().split(/\s+/);
const prog = (cmd: string) => (words(cmd)[0] ?? "").split("/").pop() ?? "";
const args = (cmd: string) => words(cmd).slice(1);
type Done = Extract<MissionEvent, { type: "done" }>;
const ok = (e: MissionEvent): e is Done => e.type === "done" && e.exitCode === 0;
const ran = (e: MissionEvent, ...names: string[]): e is Done => ok(e) && names.includes(prog(e.command));
const hasFlag = (cmd: string, letter: string) => args(cmd).some((a) => /^-[^-]/.test(a) && a.includes(letter));
const has = (cmd: string, part: string) => cmd.includes(part);
const playground = (ctx: Ctx) => `${ctx.home}/kompass-spielplatz`;
const inPlayground = (e: MissionEvent, ctx: Ctx) => e.type === "done" && e.cwd === playground(ctx);
const translated = (e: MissionEvent, test: (cmd: string) => boolean) => e.type === "translated" && test(e.command);
const tab = (e: MissionEvent, action: "new" | "switch" | "close") => e.type === "tab" && e.action === action;

export const SPIELPLATZ = "~/kompass-spielplatz";

export const MISSIONS: Mission[] = [
  // ───────────── Einstieg ─────────────
  {
    id: "orientierung", emoji: "🧭", title: ["Wo bin ich?", "Where am I?"], level: "Einstieg",
    intro: ["Das Terminal ist immer in genau einem Ordner – wie ein Finder-Fenster ohne Bilder.", "The terminal is always in exactly one folder – like a Finder window without pictures."],
    steps: [
      { id: "pwd", title: ["Frag nach deinem Standort", "Ask where you are"], text: ["pwd steht für „print working directory“ – zeig den Ordner, in dem ich gerade bin.", "pwd stands for “print working directory” – show the folder I am in."], hint: "pwd", check: (e) => ran(e, "pwd") },
      { id: "ls", title: ["Schau dich um", "Look around"], text: ["ls listet auf, was in diesem Ordner liegt.", "ls lists what is in this folder."], hint: "ls", check: (e) => ran(e, "ls") },
      { id: "ls-a", title: ["Finde das Versteckte", "Find what is hidden"], text: ["Dateien, die mit einem Punkt beginnen, sind versteckt. Die Option -a zeigt sie alle.", "Files that start with a dot are hidden. The option -a shows them all."], hint: "ls -a", check: (e) => ran(e, "ls") && hasFlag(e.command, "a") },
      { id: "ls-l", title: ["Mehr Details, bitte", "More details, please"], text: ["Mit -l bekommst du eine Liste mit Größe, Datum und Rechten. Optionen lassen sich kombinieren: -la.", "-l gives you a list with size, date and permissions. Options can be combined: -la."], hint: "ls -la", check: (e) => ran(e, "ls") && hasFlag(e.command, "l") },
    ],
  },
  {
    id: "klarsprache", emoji: "🗣️", title: ["Sag es einfach", "Just say it"], level: "Einstieg",
    intro: ["Du musst die Befehle nicht kennen: Schreib, was du willst – Kompass übersetzt es. Enter setzt den Befehl ein, ein zweites Enter führt ihn aus.", "You do not need to know the commands: write what you want – Kompass translates it. Enter inserts the command, a second Enter runs it."],
    steps: [
      { id: "zeige", title: ["Wünsch dir etwas", "Make a wish"], text: ["Tippe einen ganz normalen Satz und drück Enter.", "Type a normal sentence and press Enter."], hint: ["zeige alle dateien", "show all files"], check: (e) => translated(e, (c) => c.startsWith("ls")) },
      { id: "ausfuehren", title: ["Ausführen", "Run it"], text: ["Der Befehl steht jetzt in der Zeile. Drück noch einmal Enter.", "The command is now in the line. Press Enter once more."], check: (e) => ran(e, "ls") },
      { id: "platz", title: ["Noch eine Frage", "Another question"], text: ["Frag Kompass nach dem freien Speicherplatz.", "Ask Kompass how much disk space is free."], hint: ["wie viel speicherplatz ist frei", "how much disk space is free"], check: (e) => translated(e, (c) => c.startsWith("df")) },
      { id: "fragezeichen", title: ["Der ?-Trick", "The ? trick"], text: ["Mit einem ? am Anfang erzwingst du Klarsprache – praktisch, wenn dein Satz wie ein Befehl beginnt.", "A ? at the start forces plain language – handy when your sentence starts like a command."], hint: ["? wo bin ich", "? where am i"], check: (e) => translated(e, (c) => c === "pwd") },
    ],
  },
  {
    id: "spielplatz", emoji: "🏗️", title: ["Dein eigener Spielplatz", "Your own playground"], level: "Einstieg",
    intro: ["Wir bauen einen Ordner, in dem du nach Herzenslust ausprobieren kannst.", "Let's build a folder where you can experiment to your heart's content."],
    steps: [
      { id: "mkdir", title: ["Ordner anlegen", "Create a folder"], text: ["mkdir („make directory“) legt einen neuen Ordner an. Das ~ steht für deinen Benutzerordner.", "mkdir (“make directory”) creates a new folder. The ~ stands for your home folder."], hint: `mkdir ${SPIELPLATZ}`, check: (e) => ran(e, "mkdir") && has(e.command, "kompass-spielplatz") },
      { id: "cd", title: ["Hineingehen", "Go inside"], text: ["cd („change directory“) wechselt den Ordner. Tipp: Tippe cd ~/komp und drück Tab!", "cd (“change directory”) changes the folder. Tip: type cd ~/komp and press Tab!"], hint: `cd ${SPIELPLATZ}`, check: (e, c) => ok(e) && inPlayground(e, c) },
      { id: "touch", title: ["Eine leere Datei", "An empty file"], text: ["touch legt eine leere Datei an.", "touch creates an empty file."], hint: "touch notiz.txt", check: (e, c) => ran(e, "touch") && inPlayground(e, c) },
      { id: "echo", title: ["Etwas hineinschreiben", "Write something into it"], text: ["echo gibt Text aus, > leitet ihn in eine Datei um. Kompass fragt nach, weil > überschreibt – die Datei ist ja noch leer, also ruhig „Ja“.", "echo prints text, > redirects it into a file. Kompass asks first because > overwrites – the file is still empty, so “Yes” is fine."], hint: 'echo "Hallo Terminal" > notiz.txt', check: (e, c) => ran(e, "echo") && has(e.command, ">") && inPlayground(e, c) },
      { id: "cat", title: ["Nachlesen", "Read it back"], text: ["cat zeigt den Inhalt einer Datei.", "cat shows the content of a file."], hint: "cat notiz.txt", check: (e, c) => ran(e, "cat") && inPlayground(e, c) },
    ],
  },
  {
    id: "unterwegs", emoji: "🚶", title: ["Unterwegs", "On the move"], level: "Einstieg",
    intro: ["Drei Abkürzungen, mit denen du dich nie mehr verläufst.", "Three shortcuts so you never get lost again."],
    steps: [
      { id: "up", title: ["Eine Ebene hoch", "One level up"], text: ["Zwei Punkte .. bedeuten „der Ordner darüber“.", "Two dots .. mean “the folder above”."], hint: "cd ..", check: (e) => ran(e, "cd") && args(e.command)[0] === ".." },
      { id: "back", title: ["Zurückspringen", "Jump back"], text: ["cd - bringt dich dorthin, wo du gerade eben warst – wie der Zurück-Knopf.", "cd - takes you to where you just were – like the back button."], hint: "cd -", check: (e) => ran(e, "cd") && args(e.command)[0] === "-" },
      { id: "home", title: ["Nach Hause", "Go home"], text: ["cd ganz ohne Ziel bringt dich immer in deinen Benutzerordner.", "cd without a target always takes you to your home folder."], hint: "cd", check: (e, c) => ran(e, "cd") && e.cwd === c.home },
    ],
  },
  {
    id: "aufraeumen", emoji: "🧹", title: ["Aufräumen ohne Angst", "Tidy up without fear"], level: "Einstieg",
    intro: ["Kopieren, umbenennen, löschen – und alles wieder zurückholen.", "Copy, rename, delete – and bring everything back."],
    steps: [
      { id: "back-in", title: ["Zurück auf den Spielplatz", "Back to the playground"], text: ["Geh wieder in deinen Spielplatz-Ordner.", "Go back into your playground folder."], hint: `cd ${SPIELPLATZ}`, check: (e, c) => ok(e) && inPlayground(e, c) },
      { id: "cp", title: ["Kopie machen", "Make a copy"], text: ["cp kopiert: erst die Quelle, dann das Ziel.", "cp copies: first the source, then the target."], hint: "cp notiz.txt kopie.txt", check: (e, c) => ran(e, "cp") && inPlayground(e, c) },
      { id: "mv", title: ["Umbenennen", "Rename"], text: ["mv verschiebt – und Umbenennen ist nichts anderes als Verschieben an einen neuen Namen.", "mv moves – and renaming is just moving to a new name."], hint: "mv kopie.txt umbenannt.txt", check: (e, c) => ran(e, "mv") && inPlayground(e, c) },
      { id: "rm", title: ["Löschen", "Delete"], text: ["rm löscht. Keine Sorge: Kompass legt die Datei in den Papierkorb.", "rm deletes. Don't worry: Kompass moves the file to the Trash."], hint: "rm umbenannt.txt", check: (e, c) => ran(e, "rm", "kompass-trash") && inPlayground(e, c) },
      { id: "undo", title: ["Und wieder zurück!", "And back again!"], text: ["Klick unten rechts auf „Rückgängig“ – oder tippe kompass-undo.", "Click “Undo” at the bottom right – or type kompass-undo."], hint: "kompass-undo", check: (e) => e.type === "undo" || ran(e, "kompass-undo", "rueckgaengig") },
    ],
  },
  {
    id: "tabs", emoji: "🗂️", title: ["Mehrere Tabs", "Multiple tabs"], level: "Einstieg",
    intro: ["Wie im Browser: Mehrere Terminals nebeneinander, jedes in seinem eigenen Ordner.", "Like in a browser: several terminals side by side, each in its own folder."],
    steps: [
      { id: "new", title: ["Neuer Tab", "New tab"], text: ["Drück ⌘T. Der neue Tab startet im selben Ordner.", "Press ⌘T. The new tab starts in the same folder."], keys: "⌘T", check: (e) => tab(e, "new") },
      { id: "switch", title: ["Hin und her", "Back and forth"], text: ["Mit ⌘1, ⌘2 … springst du direkt zu einem Tab, mit ⌘⇧[ und ⌘⇧] zum vorherigen/nächsten.", "⌘1, ⌘2 … jump straight to a tab, ⌘⇧[ and ⌘⇧] go to the previous/next one."], keys: "⌘1", check: (e) => tab(e, "switch") },
      { id: "close", title: ["Tab schließen", "Close a tab"], text: ["⌘W schließt den aktuellen Tab.", "⌘W closes the current tab."], keys: "⌘W", check: (e) => tab(e, "close") },
    ],
  },

  // ───────────── Fortgeschritten ─────────────
  {
    id: "detektiv", emoji: "🔍", title: ["Detektiv-Arbeit", "Detective work"], level: "Fortgeschritten",
    intro: ["Suchen, zählen, nachschlagen – hier fängt das Terminal an, richtig Spaß zu machen.", "Search, count, look things up – this is where the terminal starts to be real fun."],
    steps: [
      { id: "grep", title: ["Text finden", "Find text"], text: ["grep sucht in Dateien nach einem Wort.", "grep searches files for a word."], hint: "grep Hallo notiz.txt", check: (e) => ran(e, "grep") },
      { id: "pipe", title: ["Befehle verbinden", "Connect commands"], text: ["Der senkrechte Strich | reicht die Ausgabe weiter. wc -l zählt die Zeilen – also: wie viele Dinge liegen hier?", "The vertical bar | passes the output on. wc -l counts lines – so: how many things are here?"], hint: "ls | wc -l", check: (e) => ok(e) && has(e.command, "|") },
      { id: "history", title: ["Dein Tagebuch", "Your diary"], text: ["history zeigt alles, was du bisher getippt hast.", "history shows everything you have typed so far."], hint: "history", check: (e) => ran(e, "history") },
      { id: "man", title: ["Die Anleitung", "The manual"], text: ["Zu fast jedem Befehl gibt es eine Anleitung. Blättern mit Leertaste, raus mit q.", "Almost every command has a manual. Space scrolls, q quits."], hint: "man ls", check: (e) => ran(e, "man") },
    ],
  },
  {
    id: "finden", emoji: "🗃️", title: ["Dateien finden", "Finding files"], level: "Fortgeschritten",
    intro: ["Mit Platzhaltern und find spürst du jede Datei auf – auch tief in Unterordnern.", "With wildcards and find you can track down any file – even deep in subfolders."],
    steps: [
      { id: "stern", title: ["Der Stern-Platzhalter", "The star wildcard"], text: ["* steht für „beliebige Zeichen“. *.txt heißt also: alles, was auf .txt endet.", "* means “any characters”. So *.txt means: everything ending in .txt."], hint: "ls *.txt", check: (e) => ran(e, "ls") && has(e.command, "*") },
      { id: "find-name", title: ["Überall suchen", "Search everywhere"], text: ["find durchsucht auch alle Unterordner. -name sagt, wonach.", "find also searches all subfolders. -name says what for."], hint: "find . -name \"*.txt\"", check: (e) => ran(e, "find") && has(e.command, "-name") },
      { id: "find-iname", title: ["Groß/Klein egal", "Ignore case"], text: ["-iname findet „Notiz.TXT“ genauso wie „notiz.txt“.", "-iname finds “Notes.TXT” just like “notes.txt”."], hint: "find ~/Desktop -iname \"*.pdf\"", check: (e) => ran(e, "find") && has(e.command, "-iname") },
      { id: "open", title: ["Im Finder zeigen", "Show in Finder"], text: ["open . öffnet den aktuellen Ordner im Finder – die Brücke zwischen Terminal und Maus.", "open . opens the current folder in Finder – the bridge between terminal and mouse."], hint: "open .", check: (e) => ran(e, "open") },
    ],
  },
  {
    id: "texte", emoji: "✍️", title: ["Texte & Notizen", "Text & notes"], level: "Fortgeschritten",
    intro: ["Texte schreiben, anhängen, durchblättern – ganz ohne Textprogramm.", "Write, append and browse text – without a text editor app."],
    steps: [
      { id: "append", title: ["Anhängen statt ersetzen", "Append instead of replace"], text: [">> hängt hinten an, statt die Datei zu überschreiben. Deshalb fragt Kompass hier nicht nach.", ">> appends instead of overwriting the file. That's why Kompass doesn't ask here."], hint: "echo \"Zweite Zeile\" >> notiz.txt", check: (e) => ran(e, "echo") && has(e.command, ">>") },
      { id: "wc", title: ["Zeilen zählen", "Count lines"], text: ["wc -l zählt die Zeilen einer Datei.", "wc -l counts the lines of a file."], hint: "wc -l notiz.txt", check: (e) => ran(e, "wc") },
      { id: "head", title: ["Nur den Anfang", "Just the beginning"], text: ["head zeigt die ersten Zeilen, -n sagt wie viele.", "head shows the first lines, -n says how many."], hint: "head -n 1 notiz.txt", check: (e) => ran(e, "head") },
      { id: "nano", title: ["Der Mini-Editor", "The mini editor"], text: ["nano ist ein Texteditor im Terminal. Schreib etwas, dann Ctrl+O (speichern), Enter, Ctrl+X (beenden).", "nano is a text editor in the terminal. Write something, then Ctrl+O (save), Enter, Ctrl+X (quit)."], hint: "nano notiz.txt", check: (e) => ran(e, "nano") },
    ],
  },
  {
    id: "filter", emoji: "🧪", title: ["Pipes & Filter", "Pipes & filters"], level: "Fortgeschritten",
    intro: ["Kleine Befehle, zusammengesteckt wie Lego – das ist die eigentliche Superkraft des Terminals.", "Small commands snapped together like Lego – the terminal's real superpower."],
    steps: [
      { id: "sort", title: ["Sortieren", "Sort"], text: ["sort bringt Zeilen in alphabetische Reihenfolge.", "sort puts lines in alphabetical order."], hint: "ls | sort -r", check: (e) => ok(e) && has(e.command, "sort") },
      { id: "grep-i", title: ["Filtern", "Filter"], text: ["grep im Rohr lässt nur passende Zeilen durch. -i macht Groß/Klein egal.", "grep in a pipe only lets matching lines through. -i ignores case."], hint: "ls -a | grep -i z", check: (e) => ok(e) && has(e.command, "|") && has(e.command, "grep") },
      { id: "history-grep", title: ["Im Verlauf suchen", "Search your history"], text: ["Welche cd-Befehle hast du schon getippt?", "Which cd commands have you typed so far?"], hint: "history | grep cd", check: (e) => ok(e) && has(e.command, "history") && has(e.command, "grep") },
      { id: "head-pipe", title: ["Die Top 5", "The top 5"], text: ["head am Ende zeigt nur die ersten Treffer.", "head at the end shows only the first results."], hint: "ls -t | head -n 5", check: (e) => ok(e) && has(e.command, "|") && has(e.command, "head") },
    ],
  },
  {
    id: "system", emoji: "💻", title: ["Blick unter die Haube", "A look under the hood"], level: "Fortgeschritten",
    intro: ["Was macht dein Mac gerade? Speicher, Programme, Version – alles nur einen Befehl entfernt.", "What is your Mac doing right now? Storage, programs, version – all just one command away."],
    steps: [
      { id: "df", title: ["Freier Speicher", "Free space"], text: ["df -h zeigt, wie voll deine Festplatte ist (-h = lesbare Größen).", "df -h shows how full your disk is (-h = readable sizes)."], hint: "df -h /", check: (e) => ran(e, "df") },
      { id: "du", title: ["Wer frisst Platz?", "What eats space?"], text: ["du -sh zeigt, wie groß ein Ordner ist.", "du -sh shows how big a folder is."], hint: "du -sh ~/Downloads", check: (e) => ran(e, "du") },
      { id: "sw_vers", title: ["Welches macOS?", "Which macOS?"], text: ["sw_vers verrät deine macOS-Version.", "sw_vers reveals your macOS version."], hint: "sw_vers", check: (e) => ran(e, "sw_vers") },
      { id: "top", title: ["Live-Monitor", "Live monitor"], text: ["top zeigt live, welche Programme arbeiten. Mit q beendest du ihn.", "top shows live which programs are working. Press q to quit."], hint: "top -o cpu", check: (e) => ran(e, "top") },
    ],
  },

  // ───────────── Profi ─────────────
  {
    id: "rechte", emoji: "🔐", title: ["Rechte verstehen", "Understanding permissions"], level: "Profi",
    intro: ["Wer darf lesen, schreiben, ausführen? Und wie wird aus einer Textdatei ein Programm?", "Who may read, write, run? And how does a text file become a program?"],
    steps: [
      { id: "ls-l", title: ["Rechte lesen", "Read permissions"], text: ["Die erste Spalte von ls -l zeigt Rechte: r = lesen, w = schreiben, x = ausführen.", "The first column of ls -l shows permissions: r = read, w = write, x = execute."], hint: "ls -l", check: (e) => ran(e, "ls") && hasFlag(e.command, "l") },
      { id: "script", title: ["Ein Mini-Programm", "A tiny program"], text: ["Wir schreiben einen Befehl in eine Datei – fertig ist ein Skript.", "We write a command into a file – and that's a script."], hint: "echo 'echo Hallo vom Skript' > hallo.sh", check: (e) => ran(e, "echo") && has(e.command, ".sh") },
      { id: "chmod", title: ["Ausführbar machen", "Make it executable"], text: ["chmod +x erlaubt das Ausführen. Kompass fragt nach, weil Rechte ändern heikel sein kann – hier ist es harmlos.", "chmod +x allows running it. Kompass asks because changing permissions can be risky – here it is harmless."], hint: "chmod +x hallo.sh", check: (e) => ran(e, "chmod") && has(e.command, "+x") },
      { id: "run", title: ["Starten!", "Run it!"], text: ["./ heißt „hier in diesem Ordner“. Damit startest du dein Skript.", "./ means “here in this folder”. That's how you start your script."], hint: "./hallo.sh", check: (e) => ok(e) && e.command.trim().startsWith("./") },
    ],
  },
  {
    id: "tricks", emoji: "⚡", title: ["Profi-Tricks", "Pro tricks"], level: "Profi",
    intro: ["Abkürzungen, die erfahrene Terminal-Nutzer jeden Tag verwenden.", "Shortcuts experienced terminal users use every day."],
    steps: [
      { id: "alias", title: ["Eigene Abkürzung", "Your own shortcut"], text: ["alias gibt einem langen Befehl einen kurzen Namen (gilt bis zum Schließen des Tabs).", "alias gives a long command a short name (lasts until you close the tab)."], hint: "alias ll='ls -lah'", check: (e) => ran(e, "alias") && has(e.command, "=") },
      { id: "use-alias", title: ["Abkürzung nutzen", "Use the shortcut"], text: ["Jetzt reicht ll.", "Now ll is enough."], hint: "ll", check: (e) => ok(e) && prog(e.command) === "ll" },
      { id: "bangbang", title: ["Nochmal, bitte", "Again, please"], text: ["!! wiederholt den letzten Befehl.", "!! repeats the last command."], hint: "!!", check: (e) => ok(e) && e.command.trim() === "!!" },
      { id: "and", title: ["Erst das, dann das", "First this, then that"], text: ["&& führt den zweiten Befehl nur aus, wenn der erste geklappt hat.", "&& runs the second command only if the first one worked."], hint: "mkdir -p test && cd test", check: (e) => ok(e) && has(e.command, "&&") },
      { id: "which", title: ["Wo wohnt ein Befehl?", "Where does a command live?"], text: ["which zeigt, welches Programm hinter einem Befehl steckt.", "which shows which program is behind a command."], hint: "which ls", check: (e) => ran(e, "which") },
    ],
  },
];

export const TOTAL_STEPS = MISSIONS.reduce((n, m) => n + m.steps.length, 0);
export const key = (m: Mission, s: Step) => `${m.id}/${s.id}`;

/** Der nächste offene Schritt einer Mission (oder undefined, wenn fertig). */
export const nextStep = (m: Mission, done: Set<string>) => m.steps.find((s) => !done.has(key(m, s)));

/** Die Mission, die gerade gespielt wird: die gewählte, sonst die erste unfertige. */
export function currentMission(chosen: string | null, done: Set<string>) {
  const pick = MISSIONS.find((m) => m.id === chosen && nextStep(m, done));
  return pick ?? MISSIONS.find((m) => nextStep(m, done));
}

/** Wie viele Missionen sind komplett geschafft? (für das Spendenfenster) */
export const completedMissions = (done: Set<string>) => MISSIONS.filter((m) => !nextStep(m, done)).length;

/**
 * Prüft ein Ereignis gegen die aktuelle Mission.
 * Nur deren nächster offener Schritt kann abgehakt werden – so bleibt alles der Reihe nach.
 */
export function evaluate(e: MissionEvent, ctx: Ctx, done: Set<string>, chosen: string | null) {
  const mission = currentMission(chosen, done);
  const step = mission && nextStep(mission, done);
  if (!mission || !step || !step.check(e, ctx)) return null;
  const after = new Set(done).add(key(mission, step));
  return { mission, step, finished: !nextStep(mission, after), after };
}

const STORE = "kompass.missions.v1";
const CHOSEN = "kompass.mission.chosen";
export function loadProgress(): Set<string> {
  try {
    return new Set(JSON.parse(localStorage.getItem(STORE) ?? "[]"));
  } catch {
    return new Set();
  }
}
export function saveProgress(done: Set<string>) {
  try {
    localStorage.setItem(STORE, JSON.stringify([...done]));
  } catch {
    /* Speichern ist nur Komfort */
  }
}
export function loadChosen(): string | null {
  try {
    return localStorage.getItem(CHOSEN);
  } catch {
    return null;
  }
}
export function saveChosen(id: string | null) {
  try {
    if (id) localStorage.setItem(CHOSEN, id);
    else localStorage.removeItem(CHOSEN);
  } catch {
    /* egal */
  }
}
