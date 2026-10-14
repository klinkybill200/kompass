/**
 * Sprachen der Oberfläche: Deutsch und Englisch.
 * Texte stehen hier zentral; Komponenten holen sie mit `const { t } = useI18n()`.
 * Platzhalter wie {name} werden beim Aufruf ersetzt: t("toast.restored1", { name }).
 */
import { createContext, ReactNode, useCallback, useContext, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export type Lang = "de" | "en";
export const LANGS: { id: Lang; label: string; name: string }[] = [
  { id: "de", label: "DE", name: "Deutsch" },
  { id: "en", label: "EN", name: "English" },
];

const de = {
  "level.safe": "Sicher",
  "level.care": "Aufpassen",
  "level.danger": "Gefährlich",
  "levelLower.safe": "sicher",
  "levelLower.care": "aufpassen",
  "levelLower.danger": "gefährlich",
  "panel.missions": "Missionen",
  "panel.trash": "Papierkorb",
  "panel.settings": "Einstellungen",
  "title.guardActive": "Wächter aktiv",
  "title.guardHint": "Der Wächter prüft jeden Befehl, bevor er ausgeführt wird.",
  "title.settings": "Einstellungen (⌘,)",
  "common.close": "Schließen",
  "tabs.close": "Tab schließen",
  "tabs.new": "Neuer Tab (⌘T)",
  "exited.text": "Die Shell in diesem Tab wurde beendet.",
  "exited.button": "Tab schließen",

  "toast.missionDone": "Mission geschafft · {n}/{total}",
  "toast.missionNext": "Stark! Such dir die nächste Mission aus.",
  "toast.stepDone": "Geschafft: {step}",
  "toast.restored1": "„{name}“ ist wieder da",
  "toast.restoredN": "{n} Elemente sind wieder da",
  "toast.restoreFailed": "Nicht alles ging zurück",
  "toast.care": "Aufpassen",
  "toast.nlLocal": "Klarsprache",
  "toast.nlKi": "KI-Übersetzung",
  "toast.nlNeedsInput": "Ergänze noch, was fehlt – dann Enter.",
  "toast.nlRun": "Enter führt den Befehl aus.",
  "toast.notUnderstood": "Nicht verstanden",
  "toast.notUnderstoodHelp": "Probier z. B. „zeige alle dateien“, „finde alle pdf dateien“ oder „wie viel platz ist frei“. Mit einem KI-Schlüssel versteht Kompass fast alles.",
  "toast.setupKi": "KI einrichten",
  "toast.whatHappened": "Was ist passiert?",
  "toast.insert": "„{cmd}“ einsetzen",
  "toast.trashed1": "{name} liegt im Papierkorb",
  "toast.trashedN": "{n} Elemente liegen im Papierkorb",
  "toast.trashedText": "Nichts ist verloren – du kannst es jederzeit zurückholen.",
  "toast.undo": "Rückgängig",

  "explain.tip": "Tipp",
  "explain.running": "Läuft",
  "explain.happens": "Das passiert",
  "explain.nl": "Klarsprache",
  "explain.ki": "KI",
  "explain.kiAsking": "Kompass fragt die KI: „{q}“",
  "explain.escCancels": "Esc bricht ab",
  "explain.kiOnEnter": "Kompass kennt das nicht direkt – beim Enter fragt es die KI.",
  "explain.notKnown": "Das versteht Kompass noch nicht. Formulier es anders oder tippe einen Befehl.",
  "explain.enterTranslates": "Enter übersetzt",
  "explain.becomes": "wird zu",
  "explain.fillIn": "danach noch ergänzen",

  "dialog.paused": "Kompass hat kurz angehalten",
  "dialog.trashTitle": "Ab in den Papierkorb?",
  "dialog.sure": "Bist du sicher?",
  "dialog.deleteForever": "Endgültig löschen",
  "dialog.cancel": "Abbrechen",
  "dialog.toTrash": "In den Papierkorb legen",
  "dialog.ratherNot": "Lieber nicht",
  "dialog.run": "Ja, ausführen",
  "dialog.keys": "Esc bricht ab · Enter wählt den hervorgehobenen Knopf",
  "dialog.folder": "Ordner",
  "dialog.entries1": "Eintrag",
  "dialog.entriesN": "Einträge",
  "dialog.file": "Datei",
  "dialog.missing": "gibt es nicht",
  "dialog.unknown": "erst beim Ausführen bekannt",
  "dialog.more": "… und {n} weitere",

  "missions.progress": "Dein Fortschritt",
  "missions.allDone": "Alles geschafft! 🎉",
  "missions.playing": "läuft",
  "missions.play": "Diese Mission spielen",
  "missions.insert": "Einsetzen ↵",
  "missions.insertTitle": "In die Eingabezeile setzen",
  "missions.reset": "Fortschritt zurücksetzen",
  "missions.resetConfirm": "Wirklich? Nochmal klicken zum Zurücksetzen",
  "missions.level.Einstieg": "Einstieg",
  "missions.level.Fortgeschritten": "Fortgeschritten",
  "missions.level.Profi": "Profi",

  "trash.loading": "Lade …",
  "trash.emptyTitle": "Noch nichts gelöscht.",
  "trash.emptyText": "Alles, was du mit rm löschst, landet im Papierkorb und taucht hier auf – mit einem Klick ist es wieder da.",
  "trash.restore": "Zurücklegen",
  "trash.gone": "nicht mehr im Papierkorb",
  "trash.from": "aus {folder}",
  "trash.today": "Heute, {time}",
  "trash.yesterday": "Gestern, {time}",

  "settings.language": "Sprache",
  "settings.languageHint": "Gilt für die ganze App – auch für Erklärungen, Wächter, Fehlerhilfe und Klarsprache.",
  "settings.nl": "Klarsprache",
  "settings.nlText": "Schreib einfach, was du willst – Kompass übersetzt es in einen Befehl. Enter setzt ihn ein, ein zweites Enter führt ihn aus. Mit ? am Anfang erzwingst du Klarsprache.",
  "settings.ki": "KI-Assistent",
  "settings.kiOn": "aktiv",
  "settings.kiOnEnv": "aktiv (Umgebungsvariable)",
  "settings.kiOff": "aus",
  "settings.kiText": "Versteht Kompass einen Wunsch nicht, kann es Claude fragen. Dafür brauchst du einen eigenen API-Schlüssel von console.anthropic.com. Jede Anfrage kostet ein paar Zehntel Cent; übertragen werden dein Satz, der aktuelle Ordnerpfad und bis zu 60 Datei- und Ordnernamen daraus.",
  "settings.kiNewKey": "Neuen Schlüssel eingeben …",
  "settings.save": "Speichern",
  "settings.saved": "Gespeichert – sicher im macOS-Schlüsselbund.",
  "settings.test": "Verbindung testen",
  "settings.testing": "Frage KI …",
  "settings.removeKey": "Schlüssel entfernen",
  "settings.removed": "Schlüssel entfernt.",
  "settings.testResult": "„zeige mir die fünf neuesten Dateien“ →",
  "settings.model": "Modell: {model}",
  "settings.font": "Schriftgröße",
  "settings.smaller": "Kleiner",
  "settings.bigger": "Größer",
  "settings.shortcuts": "Tastenkürzel",
  "settings.sc.newTab": "Neuer Tab",
  "settings.sc.closeTab": "Tab schließen",
  "settings.sc.jump": "Zu Tab springen",
  "settings.sc.prevNext": "Vorheriger / nächster Tab",
  "settings.sc.clear": "Fenster aufräumen",
  "settings.sc.panels": "Missionen · {trash} Papierkorb · {settings} Einstellungen",
  "settings.sc.basics": "Namen ergänzen · {up} letzter Befehl · {ctrlc} abbrechen",
  "settings.support": "Kompass unterstützen",
  "settings.supportText": "Kompass ist kostenlos. Wenn es dir hilft, freut sich der Entwickler über eine kleine Spende – damit das Projekt weiter gepflegt wird.",
  "settings.supportThanks": "Danke für deine Spende! ❤️",
  "settings.supportButton": "Spenden …",

  "donate.eyebrow": "Kompass ist Freeware",
  "donate.title": "Gefällt dir Kompass?",
  "donate.text": "Kompass ist und bleibt kostenlos – ohne Werbung, ohne Abo. Wenn es dir den Weg ins Terminal leichter macht, freut sich der Entwickler über eine kleine Spende. Sie hilft, das Projekt aktiv weiterzuentwickeln und zu pflegen.",
  "donate.via": "Sicher per PayPal · einmalig",
  "donate.later": "Vielleicht später",
  "donate.waitTitle": "Danke! 💚",
  "donate.waitText": "PayPal hat sich im Browser geöffnet. Hat die Spende geklappt?",
  "donate.done": "Ja, gespendet",
  "donate.notYet": "Noch nicht",
  "donate.thanksTitle": "Tausend Dank!",
  "donate.thanksText": "Deine Unterstützung hält Kompass am Leben. Dieses Fenster erscheint nicht mehr.",
  "donate.ok": "Gern geschehen",

  "tips.0": "Tippe einfach los – Kompass erklärt hier jeden Befehl, noch bevor du Enter drückst.",
  "tips.1": "Du kannst auch in normalen Sätzen schreiben: „zeige alle pdf dateien“ – Kompass übersetzt.",
  "tips.2": "Die Tab-Taste ergänzt Datei- und Ordnernamen automatisch. Spart Tipparbeit und Tippfehler!",
  "tips.3": "⌘T öffnet einen neuen Tab im selben Ordner.",
  "tips.4": "Pfeil ↑ holt deine letzten Befehle zurück.",
  "tips.5": "Ctrl+C bricht einen laufenden Befehl ab – falls mal etwas hängt.",
  "tips.6": "Gelöscht ist bei Kompass nichts endgültig: rm legt alles in den Papierkorb.",
  "tips.7": "„open .“ öffnet den aktuellen Ordner im Finder.",
  "tips.8": "Beginnt dein Satz wie ein Befehl? Ein ? davor erzwingt Klarsprache.",
};

type Key = keyof typeof de;

const en: Record<Key, string> = {
  "level.safe": "Safe",
  "level.care": "Careful",
  "level.danger": "Dangerous",
  "levelLower.safe": "safe",
  "levelLower.care": "careful",
  "levelLower.danger": "dangerous",
  "panel.missions": "Missions",
  "panel.trash": "Trash",
  "panel.settings": "Settings",
  "title.guardActive": "Guard active",
  "title.guardHint": "The guard checks every command before it runs.",
  "title.settings": "Settings (⌘,)",
  "common.close": "Close",
  "tabs.close": "Close tab",
  "tabs.new": "New tab (⌘T)",
  "exited.text": "The shell in this tab has ended.",
  "exited.button": "Close tab",

  "toast.missionDone": "Mission complete · {n}/{total}",
  "toast.missionNext": "Great! Pick your next mission.",
  "toast.stepDone": "Done: {step}",
  "toast.restored1": "“{name}” is back",
  "toast.restoredN": "{n} items are back",
  "toast.restoreFailed": "Not everything could be restored",
  "toast.care": "Careful",
  "toast.nlLocal": "Plain language",
  "toast.nlKi": "AI translation",
  "toast.nlNeedsInput": "Fill in what is missing – then press Enter.",
  "toast.nlRun": "Enter runs the command.",
  "toast.notUnderstood": "Not understood",
  "toast.notUnderstoodHelp": "Try e.g. “show all files”, “find all pdf files” or “how much disk space is free”. With an AI key Kompass understands almost anything.",
  "toast.setupKi": "Set up AI",
  "toast.whatHappened": "What happened?",
  "toast.insert": "Insert “{cmd}”",
  "toast.trashed1": "{name} is in the Trash",
  "toast.trashedN": "{n} items are in the Trash",
  "toast.trashedText": "Nothing is lost – you can bring it back at any time.",
  "toast.undo": "Undo",

  "explain.tip": "Tip",
  "explain.running": "Running",
  "explain.happens": "What happens",
  "explain.nl": "Plain language",
  "explain.ki": "AI",
  "explain.kiAsking": "Kompass is asking the AI: “{q}”",
  "explain.escCancels": "Esc cancels",
  "explain.kiOnEnter": "Kompass does not know this directly – Enter asks the AI.",
  "explain.notKnown": "Kompass does not understand this yet. Rephrase it or type a command.",
  "explain.enterTranslates": "Enter translates",
  "explain.becomes": "becomes",
  "explain.fillIn": "fill in the rest afterwards",

  "dialog.paused": "Kompass paused for a moment",
  "dialog.trashTitle": "Move to the Trash?",
  "dialog.sure": "Are you sure?",
  "dialog.deleteForever": "Delete permanently",
  "dialog.cancel": "Cancel",
  "dialog.toTrash": "Move to Trash",
  "dialog.ratherNot": "Better not",
  "dialog.run": "Yes, run it",
  "dialog.keys": "Esc cancels · Enter picks the highlighted button",
  "dialog.folder": "Folder",
  "dialog.entries1": "item",
  "dialog.entriesN": "items",
  "dialog.file": "File",
  "dialog.missing": "does not exist",
  "dialog.unknown": "known only when it runs",
  "dialog.more": "… and {n} more",

  "missions.progress": "Your progress",
  "missions.allDone": "All done! 🎉",
  "missions.playing": "active",
  "missions.play": "Play this mission",
  "missions.insert": "Insert ↵",
  "missions.insertTitle": "Put it into the input line",
  "missions.reset": "Reset progress",
  "missions.resetConfirm": "Really? Click again to reset",
  "missions.level.Einstieg": "Getting started",
  "missions.level.Fortgeschritten": "Intermediate",
  "missions.level.Profi": "Pro",

  "trash.loading": "Loading …",
  "trash.emptyTitle": "Nothing deleted yet.",
  "trash.emptyText": "Everything you delete with rm goes to the Trash and shows up here – one click brings it back.",
  "trash.restore": "Put back",
  "trash.gone": "no longer in the Trash",
  "trash.from": "from {folder}",
  "trash.today": "Today, {time}",
  "trash.yesterday": "Yesterday, {time}",

  "settings.language": "Language",
  "settings.languageHint": "Applies to the whole app – including explanations, the guard, error help and plain language.",
  "settings.nl": "Plain language",
  "settings.nlText": "Just write what you want – Kompass turns it into a command. Enter inserts it, a second Enter runs it. Start with ? to force plain language.",
  "settings.ki": "AI assistant",
  "settings.kiOn": "on",
  "settings.kiOnEnv": "on (environment variable)",
  "settings.kiOff": "off",
  "settings.kiText": "If Kompass does not understand a request, it can ask Claude. You need your own API key from console.anthropic.com. Each request costs a fraction of a cent; your sentence, the current folder path and up to 60 file and folder names from it are sent.",
  "settings.kiNewKey": "Enter a new key …",
  "settings.save": "Save",
  "settings.saved": "Saved – safely in the macOS keychain.",
  "settings.test": "Test connection",
  "settings.testing": "Asking the AI …",
  "settings.removeKey": "Remove key",
  "settings.removed": "Key removed.",
  "settings.testResult": "“show me the five newest files” →",
  "settings.model": "Model: {model}",
  "settings.font": "Font size",
  "settings.smaller": "Smaller",
  "settings.bigger": "Bigger",
  "settings.shortcuts": "Keyboard shortcuts",
  "settings.sc.newTab": "New tab",
  "settings.sc.closeTab": "Close tab",
  "settings.sc.jump": "Jump to tab",
  "settings.sc.prevNext": "Previous / next tab",
  "settings.sc.clear": "Clear the window",
  "settings.sc.panels": "Missions · {trash} Trash · {settings} Settings",
  "settings.sc.basics": "complete names · {up} last command · {ctrlc} cancel",
  "settings.support": "Support Kompass",
  "settings.supportText": "Kompass is free. If it helps you, the developer appreciates a small donation – so the project can keep being maintained.",
  "settings.supportThanks": "Thank you for your donation! ❤️",
  "settings.supportButton": "Donate …",

  "donate.eyebrow": "Kompass is freeware",
  "donate.title": "Enjoying Kompass?",
  "donate.text": "Kompass is and stays free – no ads, no subscription. If it makes the terminal easier for you, the developer would love a small donation. It helps keep the project actively developed and maintained.",
  "donate.via": "Secure via PayPal · one-time",
  "donate.later": "Maybe later",
  "donate.waitTitle": "Thank you! 💚",
  "donate.waitText": "PayPal opened in your browser. Did the donation go through?",
  "donate.done": "Yes, I donated",
  "donate.notYet": "Not yet",
  "donate.thanksTitle": "Thanks a million!",
  "donate.thanksText": "Your support keeps Kompass alive. This window will not appear again.",
  "donate.ok": "You're welcome",

  "tips.0": "Just start typing – Kompass explains every command here before you press Enter.",
  "tips.1": "You can also write plain sentences: “find all pdf files” – Kompass translates.",
  "tips.2": "The Tab key completes file and folder names. Saves typing and typos!",
  "tips.3": "⌘T opens a new tab in the same folder.",
  "tips.4": "The ↑ arrow brings back your last commands.",
  "tips.5": "Ctrl+C cancels a running command – in case something hangs.",
  "tips.6": "Nothing is deleted for good in Kompass: rm moves everything to the Trash.",
  "tips.7": "“open .” opens the current folder in Finder.",
  "tips.8": "Does your sentence start like a command? Put a ? in front to force plain language.",
};

const DICTS: Record<Lang, Record<Key, string>> = { de, en };
export const TIP_COUNT = 9;

export type T = (key: Key, vars?: Record<string, string | number>) => string;
export type TKey = Key;

/** Zweisprachiger Text direkt im Code: [deutsch, englisch]. */
export type L = [string, string];

type Ctx = { lang: Lang; setLang: (l: Lang) => void; t: T; l: (text: L | string) => string };
const I18n = createContext<Ctx | null>(null);

function savedLang(): Lang | null {
  try {
    const saved = localStorage.getItem("kompass.lang");
    if (saved === "de" || saved === "en") return saved;
  } catch {
    /* egal */
  }
  return null;
}

function initialLang(): Lang {
  return savedLang() ?? (navigator.language.toLowerCase().startsWith("de") ? "de" : "en");
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>(initialLang);

  // Noch nichts gewählt? Dann der Systemsprache des Macs folgen (fragt Rust, siehe oben).
  useEffect(() => {
    if (savedLang() || !("__TAURI_INTERNALS__" in window)) return;
    invoke<string>("system_language")
      .then((l) => { if (l === "de" || l === "en") setLangState(l); })
      .catch(() => {});
  }, []);

  // Dem Backend Bescheid geben (Wächter, Erklärungen, Fehlerhilfe sprechen dann dieselbe Sprache).
  useEffect(() => {
    document.documentElement.lang = lang;
    if ("__TAURI_INTERNALS__" in window) void invoke("set_language", { lang });
  }, [lang]);

  const setLang = useCallback((l: Lang) => {
    setLangState(l);
    try { localStorage.setItem("kompass.lang", l); } catch { /* egal */ }
  }, []);

  const value = useMemo<Ctx>(() => {
    const dict = DICTS[lang];
    const t: T = (key, vars) => {
      let s = dict[key] ?? de[key] ?? key;
      if (vars) for (const [k, v] of Object.entries(vars)) s = s.split(`{${k}}`).join(String(v));
      return s;
    };
    const l = (text: L | string) => (typeof text === "string" ? text : lang === "en" ? text[1] : text[0]);
    return { lang, setLang, t, l };
  }, [lang, setLang]);

  return <I18n.Provider value={value}>{children}</I18n.Provider>;
}

export function useI18n() {
  const c = useContext(I18n);
  if (!c) throw new Error("useI18n außerhalb von I18nProvider");
  return c;
}
