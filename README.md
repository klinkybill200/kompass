<p align="center">
  <img src="website/assets/icon-512.png" width="96" height="96" alt="Kompass icon" />
</p>

<h1 align="center">Kompass</h1>

<p align="center">
  <b>The terminal that has your back.</b><br />
  A free, friendly terminal for macOS – for people who are afraid of the terminal, and for pros who want to move faster.
</p>

<p align="center">
  <a href="https://kompass-rust.vercel.app">Website</a> ·
  <a href="https://github.com/klinkybill200/kompass/releases/latest">Download</a> ·
  <a href="https://kompass-rust.vercel.app/#support">Support the project</a> ·
  <a href="https://kompass-rust.vercel.app/?lang=de">Deutsch</a>
</p>

---

## What it does

Kompass runs a **real zsh** (with your own `.zshrc`) and adds a friendly co-pilot on top:

- 🛡️ **Command guard** – every line is checked *before* it runs and rated safe, careful or dangerous. Risky commands (`rm`, `sudo`, `chmod`, `>` on existing files, `curl | sh`, …) get a dialog that explains in plain words what will happen.
- 🗑️ **`rm` goes to the Trash** – deletions land in the macOS Trash and can be undone with one click (or `kompass-undo`).
- 💬 **Plain language → command** – type *“find all pdf files”* or *“wie viel speicherplatz ist frei”*; Kompass inserts the matching command and only runs it on your second Enter. Works offline with built-in rules; optionally falls back to Claude with your own API key.
- 🔎 **Live explanations** – while you type, a bar explains every part of the command (flags, files, pipes, redirects).
- 🩹 **Error help** – cryptic errors are translated, typos detected, corrected commands offered with one click.
- 🧭 **13 learning missions** – from “Where am I?” to scripts and pipes, checked against the commands you really ran.
- 🗂️ **Tabs & native shortcuts** – ⌘T, ⌘W, ⌘1–9, ⌘K …
- 🌍 **English & German** – UI, guard, explanations and plain language.

Everything runs locally. No account, no analytics, no tracking.

## Download & install

Grab the latest `.dmg` from the [releases page](https://github.com/klinkybill200/kompass/releases/latest) or the [website](https://kompass-rust.vercel.app/#download).

Requirements: macOS 11 or later, Apple Silicon (M1–M4).

1. Open the `.dmg` and drag **Kompass** into **Applications**.
2. Kompass is not notarized by Apple yet, so macOS warns on first launch.
3. Open **System Settings → Privacy & Security**, scroll down and click **“Open Anyway”** (only once).
   Alternatively: `xattr -dr com.apple.quarantine /Applications/Kompass.app`

## Build from source

You need [Rust](https://www.rust-lang.org/tools/install) (stable), Node.js 20+ and the Xcode command line tools.

```bash
git clone https://github.com/klinkybill200/kompass.git
cd kompass
npm install
npm run tauri dev      # run in development mode
npx tauri build        # build Kompass.app (src-tauri/target/release/bundle/macos/)
```

Tests (Rust backend, incl. an end-to-end test with a real zsh):

```bash
cd src-tauri && cargo test
```

Check the UI without the Rust backend: `npm run dev` and open `http://localhost:1420/#danger` in a browser – `src/devmock.ts` simulates the backend (scenes: `#live #care #danger #trash #help #mission #nl #ki`).

## How it works

Kompass starts zsh in a pseudo terminal (`portable-pty`) with a small shell integration (`src-tauri/shell/kompass.zsh`). The Enter key is redirected: before a line runs, zsh sends it invisibly to Kompass, waits for the answer and then runs, rewrites or cancels it.

| File | Purpose |
|---|---|
| `src-tauri/src/pty.rs` | one zsh per tab, bridge shell ↔ guard / plain language ↔ window |
| `src-tauri/src/guard.rs` | command guard: parses the line, rates it (safe / careful / dangerous) |
| `src-tauri/src/nl.rs` | plain language → command (local rules, recognizes real file names) |
| `src-tauri/src/ki.rs` | optional AI fallback via the Claude API (key stored in the macOS keychain) |
| `src-tauri/src/explain.rs` | live explanations: command lexicon (DE/EN), typo suggestions |
| `src-tauri/src/errors.rs` | error translator with suggested fixes |
| `src-tauri/src/trashlog.rs`, `trash_cli.rs` | Trash with undo log, `kompass-trash` / `kompass-undo` |
| `src-tauri/src/i18n.rs`, `src/i18n.tsx` | languages (backend messages and UI) |
| `src-tauri/src/lib.rs` | app start and native menu |
| `src/` | React UI: terminal (xterm.js), explanation bar, dialogs, missions, settings |
| `website/` | the static website (deployed on Vercel) |

Messages from the shell to Kompass are invisible OSC sequences (`US` = byte 0x1F):

| Sequence | Meaning |
|---|---|
| `ESC ]7770; base64(cmd);base64(cwd);known BEL` | check before running – reply `r.` run / `r<b64>.` run rewritten / `e<b64>.` insert command (plain language) / `n<b64>.` not understood / `c<b64>.` cancel |
| `ESC ]7771; cwd US known US line BEL` | live input while typing |
| `ESC ]7772; exit code US cwd BEL` | command finished |
| `ESC ]7773; batch BEL` | `kompass-trash` moved files to the Trash |

> Code comments are written in German – the project started as a learning project for someone new to Rust.

## Contributing

Ideas, bug reports and pull requests are very welcome – please open an [issue](https://github.com/klinkybill200/kompass/issues) first for bigger changes.

## Support

Kompass is freeware. If it helps you, a small [donation via PayPal](https://kompass-rust.vercel.app/#support) helps keep it maintained. 💚

## License

[MIT](LICENSE) © 2026 Andreas Klingenberger
