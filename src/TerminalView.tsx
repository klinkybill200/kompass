import { useEffect, useRef } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { ServerMsg, closeShell, resizeShell, spawnShell, writeShell } from "./shell";

/** Was App von außen mit einem Terminal machen kann. */
export type TermHandle = { focus: () => void; clear: () => void; setFontSize: (px: number) => void };

type Props = {
  sid: number;
  active: boolean;
  startCwd: string | null;
  fontSize: number;
  onMessage: (sid: number, m: ServerMsg) => void;
  /** App merkt sich hier pro Tab, wie das Terminal fokussiert/geleert wird. */
  register: (sid: number, h: TermHandle | null) => void;
};

const THEME = {
  background: "#11151d",
  foreground: "#e6eaf2",
  cursor: "#6ee7b7",
  cursorAccent: "#11151d",
  selectionBackground: "#6ee7b744",
  black: "#1b2130", brightBlack: "#5c667a",
  red: "#f87171", brightRed: "#fca5a5",
  green: "#6ee7b7", brightGreen: "#a7f3d0",
  yellow: "#fbbf24", brightYellow: "#fde68a",
  blue: "#60a5fa", brightBlue: "#93c5fd",
  magenta: "#c084fc", brightMagenta: "#d8b4fe",
  cyan: "#22d3ee", brightCyan: "#67e8f9",
  white: "#e6eaf2", brightWhite: "#ffffff",
};

export default function TerminalView({ sid, active, startCwd, fontSize, onMessage, register }: Props) {
  const host = useRef<HTMLDivElement>(null);
  const handlers = useRef({ onMessage, register });
  handlers.current = { onMessage, register };
  const fitRef = useRef<() => void>(() => {});
  const termRef = useRef<Terminal | null>(null);

  useEffect(() => {
    const term = new Terminal({
      fontFamily: '"SF Mono", Menlo, Monaco, monospace',
      fontSize,
      lineHeight: 1.25,
      cursorBlink: true,
      cursorStyle: "bar",
      allowProposedApi: true,
      macOptionIsMeta: false,
      theme: THEME,
    });
    termRef.current = term;
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(host.current!);
    // ⌘-Tastenkürzel gehören dem Mac-Menü (⌘T, ⌘W …), nicht dem Terminal.
    term.attachCustomKeyEventHandler((e) => !e.metaKey);
    const safeFit = () => {
      // Unsichtbare Tabs haben keine Größe – dann nicht messen.
      if (host.current && host.current.offsetWidth > 0) fit.fit();
    };
    fitRef.current = safeFit;
    safeFit();

    handlers.current.register(sid, {
      focus: () => term.focus(),
      clear: () => term.clear(),
      setFontSize: (px) => { term.options.fontSize = px; safeFit(); },
    });

    spawnShell(sid, term.cols, term.rows, startCwd, (m) => {
      if (m.kind === "output") term.write(m.data);
      handlers.current.onMessage(sid, m);
    }).catch((e) => term.write(`\r\n\x1b[31mFehler: ${e}\x1b[0m\r\n`));

    const input = term.onData((d) => void writeShell(sid, d));
    const resized = term.onResize(({ cols, rows }) => void resizeShell(sid, cols, rows));
    const ro = new ResizeObserver(safeFit);
    ro.observe(host.current!);

    return () => {
      ro.disconnect();
      input.dispose();
      resized.dispose();
      term.dispose();
      handlers.current.register(sid, null);
      void closeShell(sid);
    };
    // Bewusst nur beim ersten Anzeigen: jeder Tab startet genau eine Shell.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sid]);

  // Tab wird sichtbar → neu messen und Tastatur hierher.
  useEffect(() => {
    if (!active) return;
    const t = requestAnimationFrame(() => {
      fitRef.current();
      termRef.current?.focus();
    });
    return () => cancelAnimationFrame(t);
  }, [active]);

  return <div className={`terminal-host ${active ? "" : "inactive"}`} ref={host} />;
}
