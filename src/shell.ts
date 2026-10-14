import { Channel, invoke } from "@tauri-apps/api/core";

export type Level = "safe" | "care" | "danger";

export type Finding = { level: Level; title: string; text: string };
export type Target = { name: string; kind: "file" | "dir" | "missing" | "unknown"; entries: number | null };
export type Verdict = {
  level: Level;
  findings: Finding[];
  trash: { rewritten: string; targets: Target[] } | null;
};
export type Part = { token: string; meaning: string; kind: "cmd" | "flag" | "arg" | "op" | "redir" | "unknown" };
export type Explanation = { summary: string; parts: Part[]; level: Level };
export type Help = { title: string; text: string; suggestion: string | null };
export type TrashItem = { name: string; folder: string; isDir: boolean; available: boolean };
export type TrashBatch = { batch: number; items: TrashItem[] };
export type Restored = { name: string; ok: boolean; message: string };
export type Suggestion = { command: string; text: string; needsInput: boolean; source: "local" | "ki" };
export type NlLive = { suggestion: Suggestion | null; kiAvailable: boolean };
export type KiStatus = { configured: boolean; source: "keychain" | "env" | "none"; model: string };

/** Nachrichten vom Rust-Backend (siehe ServerMsg in pty.rs). Jeder Tab hat seinen eigenen Kanal. */
export type ServerMsg =
  | { kind: "output"; data: string }
  | { kind: "verdict"; id: number; command: string; verdict: Verdict; needsConfirm: boolean }
  | { kind: "live"; line: string; explanation: Explanation | null; nl: NlLive | null }
  | { kind: "translated"; query: string; suggestion: Suggestion }
  | { kind: "thinking"; id: number; query: string }
  | { kind: "notUnderstood"; query: string; reason: string; kiAvailable: boolean }
  | { kind: "done"; command: string; exitCode: number; cwd: string }
  | { kind: "help"; command: string; help: Help }
  | { kind: "trashed"; batch: number; items: TrashItem[] }
  | { kind: "exit" };

export type Choice = "trash" | "run" | "cancel";

export function spawnShell(sid: number, cols: number, rows: number, cwd: string | null, onMsg: (m: ServerMsg) => void) {
  const channel = new Channel<ServerMsg>();
  channel.onmessage = onMsg;
  return invoke("pty_spawn", { onEvent: channel, sid, cols, rows, cwd });
}

export const writeShell = (sid: number, data: string) => invoke("pty_write", { sid, data });
export const resizeShell = (sid: number, cols: number, rows: number) => invoke("pty_resize", { sid, cols, rows });
export const closeShell = (sid: number) => invoke("pty_close", { sid });
export const answerGuard = (sid: number, id: number, choice: Choice) => invoke("guard_answer", { sid, id, choice });
export const trashList = () => invoke<TrashBatch[]>("trash_list");
export const trashRestore = (batch: number) => invoke<Restored[]>("trash_restore", { batch });
export const kiStatus = () => invoke<KiStatus>("ki_status");
export const kiSetKey = (key: string) => invoke<KiStatus>("ki_set_key", { key });
export const kiClearKey = () => invoke<KiStatus>("ki_clear_key");
export const kiTest = () => invoke<Suggestion>("ki_test");

/** Setzt einen Befehl in die Eingabezeile (ohne ihn auszuführen). Ctrl+U leert die Zeile vorher. */
export const insertCommand = (sid: number, cmd: string) => writeShell(sid, "\x15" + cmd);

export const isTauri = () => "__TAURI_INTERNALS__" in window;
