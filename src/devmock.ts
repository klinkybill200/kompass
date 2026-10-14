/**
 * NUR für die Entwicklung: Simuliert das Rust-Backend, wenn die Oberfläche
 * im normalen Browser (ohne Tauri) geöffnet wird – zum Prüfen des Designs.
 * Szenario per URL-Hash: #live, #care, #danger, #trash, #help, #mission, #nl, #ki
 * Gelangt nicht in die fertige App (siehe main.tsx).
 */
import { mockIPC } from "@tauri-apps/api/mocks";
import type { ServerMsg } from "./shell";

const PROMPT = "\x1b[32mandreas\x1b[0m@Mac \x1b[34m~/kompass-spielplatz\x1b[0m % ";

const scenarios: Record<string, ServerMsg[]> = {
  live: [
    { kind: "output", data: "ls -la Fotos" },
    { kind: "live", line: "ls -la Fotos", explanation: { summary: "Zeigt, was in „Fotos“ liegt – als Liste mit Details, auch versteckte Dateien.", level: "safe", parts: [
      { token: "ls", meaning: "Zeigt, was im aktuellen Ordner liegt", kind: "cmd" },
      { token: "-la", meaning: "als Liste mit Details, auch versteckte Dateien", kind: "flag" },
      { token: "Fotos", meaning: "Ordner", kind: "arg" },
    ] }, nl: null },
  ],
  care: [
    { kind: "verdict", id: 1, command: "cp notiz.txt Fotos/", needsConfirm: false, verdict: { level: "care", trash: null, findings: [{ level: "care", title: "Überschreibt Dateien", text: "Kopiert nach „Fotos/“. Dabei wird „Fotos/notiz.txt“ überschrieben – der alte Inhalt ist danach weg." }] } },
  ],
  danger: [
    { kind: "output", data: "rm -r Fotos *.jpg" },
    { kind: "verdict", id: 2, command: "rm -r Fotos *.jpg", needsConfirm: true, verdict: { level: "danger", findings: [{ level: "danger", title: "Dateien löschen", text: "Löscht 2 Dateien und 1 Ordner. Ordner werden samt ihrem kompletten Inhalt gelöscht. Normalerweise ist das endgültig. Kompass legt die Dateien stattdessen in den Papierkorb – du kannst sie jederzeit zurückholen." }],
      trash: { rewritten: "kompass-trash -r Fotos *.jpg", targets: [{ name: "Fotos", kind: "dir", entries: 12 }, { name: "a.jpg", kind: "file", entries: null }, { name: "b.jpg", kind: "file", entries: null }] } } },
  ],
  trash: [
    { kind: "output", data: "\r\n\x1b[32m🗑  Datei „umbenannt.txt“ liegt jetzt im Papierkorb.\x1b[0m\r\n" + PROMPT },
    { kind: "trashed", batch: Date.now(), items: [{ name: "umbenannt.txt", folder: "/Users/demo/kompass-spielplatz", isDir: false, available: true }] },
  ],
  help: [
    { kind: "output", data: "cd fotos\r\ncd: no such file or directory: fotos\r\n" + PROMPT },
    { kind: "help", command: "cd fotos", help: { title: "Nicht gefunden", text: "„fotos“ gibt es hier nicht. Es gibt aber „Fotos“ – meintest du das? (Groß- und Kleinschreibung zählt!)", suggestion: "cd Fotos" } },
  ],
  nl: [
    { kind: "output", data: "finde alle pdf dateien" },
    { kind: "live", line: "finde alle pdf dateien", explanation: null, nl: { kiAvailable: false, suggestion: { command: "find . -type f -iname '*.pdf'", text: "Findet alle PDF-Dateien hier und in allen Unterordnern", needsInput: false, source: "local" } } },
  ],
  ki: [
    { kind: "output", data: "fasse die rechnungen von märz zusammen" },
    { kind: "thinking", id: 9, query: "fasse die rechnungen von märz zusammen" },
  ],
  mission: [
    { kind: "done", command: "pwd", exitCode: 0, cwd: "/Users/demo" },
    { kind: "done", command: "ls", exitCode: 0, cwd: "/Users/demo" },
  ],
};

export function install() {
  const sends = new Map<number, (m: ServerMsg) => void>();
  mockIPC((cmd, args) => {
    const a = args as Record<string, unknown>;
    switch (cmd) {
      case "pty_spawn": {
        const ch = a.onEvent as { onmessage: (m: ServerMsg) => void };
        const sid = a.sid as number;
        const send = (m: ServerMsg) => ch.onmessage(m);
        sends.set(sid, send);
        setTimeout(() => {
          send({ kind: "output", data: `\x1b[2mWillkommen bei Kompass (Vorschau-Modus, Tab ${sid})\x1b[0m\r\n\r\n` + PROMPT });
          if (sid !== 1) return;
          const steps = scenarios[location.hash.slice(1)] ?? [];
          steps.forEach((m, i) => setTimeout(() => send(m), 300 + i * 200));
        }, 200);
        return null;
      }
      case "trash_list":
        return [{ batch: Date.now() - 600000, items: [{ name: "umbenannt.txt", folder: "/Users/demo/kompass-spielplatz", isDir: false, available: true }, { name: "Alte Fotos", folder: "/Users/demo/Desktop", isDir: true, available: true }] }];
      case "trash_restore":
        return [{ name: "umbenannt.txt", ok: true, message: "ist wieder da" }];
      case "ki_status":
        return { configured: false, source: "none", model: "claude-opus-5-5" };
      case "set_language":
      case "open_donation":
        return null;
      case "plugin:path|resolve_directory":
        return "/Users/demo";
      default:
        return null;
    }
  });
  // Für Klick-Tests im Browser: window.kompassSend({...}, tab)
  (window as unknown as { kompassSend: (m: ServerMsg, sid?: number) => void }).kompassSend = (m, sid = 1) => sends.get(sid)?.(m);
}
