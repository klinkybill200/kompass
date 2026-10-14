import { useCallback, useEffect, useRef, useState } from "react";
import { homeDir } from "@tauri-apps/api/path";
import { listen } from "@tauri-apps/api/event";
import TerminalView, { TermHandle } from "./TerminalView";
import TabBar, { Tab } from "./TabBar";
import ConfirmDialog from "./ConfirmDialog";
import ExplainBar from "./ExplainBar";
import Toasts, { ToastItem } from "./Toasts";
import MissionsPanel from "./MissionsPanel";
import TrashPanel from "./TrashPanel";
import SettingsPanel from "./SettingsPanel";
import { Choice, Explanation, Level, NlLive, ServerMsg, Verdict, answerGuard, insertCommand, isTauri, trashRestore } from "./shell";
import { MissionEvent, TOTAL_STEPS, completedMissions, evaluate, loadChosen, loadProgress, saveChosen, saveProgress } from "./missions";
import { LANGS, useI18n } from "./i18n";
import DonateDialog from "./DonateDialog";
import { countCommand, shouldAsk } from "./donate";

type Panel = "missions" | "trash" | "settings" | null;

/** Was jeder Tab für die Oberfläche braucht. */
type TabUi = {
  live: { line: string; explanation: Explanation | null; nl: NlLive | null };
  running: boolean;
  last: { level: Level; command: string } | null;
  pending: { id: number; command: string; verdict: Verdict } | null;
  thinking: { id: number; query: string } | null;
  exited: boolean;
};
const emptyUi = (): TabUi => ({ live: { line: "", explanation: null, nl: null }, running: false, last: null, pending: null, thinking: null, exited: false });

let toastId = 1;
let nextSid = 1;

function firstPanel(): Panel {
  try {
    if (localStorage.getItem("kompass.seen")) return null;
    localStorage.setItem("kompass.seen", "1");
  } catch {
    /* egal */
  }
  return "missions"; // beim allerersten Start gleich die Missionen zeigen
}

function loadFont(): number {
  try {
    const n = Number(localStorage.getItem("kompass.font"));
    return n >= 10 && n <= 28 ? n : 14;
  } catch {
    return 14;
  }
}

const shortPath = (p: string, home: string) => (home && p.startsWith(home) ? "~" + p.slice(home.length) : p);
const titleFor = (cwd: string | null, home: string) => {
  if (!cwd || cwd === home) return "~";
  return cwd.split("/").filter(Boolean).pop() ?? "/";
};

export default function App() {
  const { t, l, lang, setLang } = useI18n();
  const tRef = useRef({ t, l });
  tRef.current = { t, l };
  const [donateOpen, setDonateOpen] = useState(false);
  const donateAsked = useRef(false); // höchstens einmal pro App-Start fragen
  const [tabs, setTabs] = useState<Tab[]>(() => [{ id: nextSid++, title: "~", cwd: null, busy: null }]);
  const [active, setActive] = useState(tabs[0].id);
  const [ui, setUi] = useState<Record<number, TabUi>>({});
  const [toasts, setToasts] = useState<ToastItem[]>([]);
  const [panel, setPanel] = useState<Panel>(firstPanel);
  const [trashKey, setTrashKey] = useState(0);
  const [progress, setProgress] = useState<Set<string>>(loadProgress);
  const [chosen, setChosen] = useState<string | null>(loadChosen);
  const [home, setHome] = useState("");
  const [fontSize, setFontSize] = useState(loadFont);

  // Refs: immer der aktuelle Stand, auch in Event-Handlern, die nur einmal angemeldet werden.
  const handles = useRef(new Map<number, TermHandle>());
  const st = useRef({ tabs, active, progress, chosen, home, ui, fontSize });
  st.current = { tabs, active, progress, chosen, home, ui, fontSize };

  useEffect(() => {
    homeDir().then((h) => setHome(h.replace(/\/$/, ""))).catch(() => {});
  }, []);

  const tabUi = (id: number) => ui[id] ?? emptyUi();
  const patchUi = useCallback((id: number, patch: Partial<TabUi> | ((u: TabUi) => Partial<TabUi>)) => {
    setUi((all) => {
      const cur = all[id] ?? emptyUi();
      return { ...all, [id]: { ...cur, ...(typeof patch === "function" ? patch(cur) : patch) } };
    });
  }, []);
  const patchTab = useCallback((id: number, patch: Partial<Tab>) => {
    setTabs((all) => all.map((t) => (t.id === id ? { ...t, ...patch } : t)));
  }, []);

  const focusActive = useCallback(() => {
    setTimeout(() => handles.current.get(st.current.active)?.focus(), 0);
  }, []);

  const pushToast = useCallback((t: Omit<ToastItem, "id">) => {
    const id = toastId++;
    // Höchstens 3 Karten gleichzeitig – die älteste fliegt raus.
    setToasts((all) => [...all.slice(-2), { ...t, id }]);
    return id;
  }, []);
  const closeToast = useCallback((id: number) => setToasts((all) => all.filter((t) => t.id !== id)), []);

  const insert = useCallback((cmd: string) => {
    void insertCommand(st.current.active, cmd);
    focusActive();
  }, [focusActive]);

  // ── Missionen ──
  const missionEvent = useCallback((e: MissionEvent) => {
    const r = evaluate(e, { home: st.current.home }, st.current.progress, st.current.chosen);
    if (!r) return;
    st.current.progress = r.after;
    setProgress(r.after);
    saveProgress(r.after);
    const { t, l } = tRef.current;
    if (r.finished) {
      pushToast({ kind: "mission", eyebrow: t("toast.missionDone", { n: r.after.size, total: TOTAL_STEPS }), title: `${r.mission.emoji} ${l(r.mission.title)}`, text: t("toast.missionNext"), timeout: 8000 });
    } else {
      pushToast({ kind: "success", title: t("toast.stepDone", { step: l(r.step.title) }), timeout: 3500 });
    }
  }, [pushToast]);

  const chooseMission = useCallback((id: string) => {
    setChosen(id);
    st.current.chosen = id;
    saveChosen(id);
    focusActive();
  }, [focusActive]);

  // ── Papierkorb ──
  const restore = useCallback(async (batch: number) => {
    const results = await trashRestore(batch);
    const good = results.filter((r) => r.ok);
    const bad = results.filter((r) => !r.ok);
    const { t } = tRef.current;
    if (good.length) {
      pushToast({ kind: "success", title: good.length === 1 ? t("toast.restored1", { name: good[0].name }) : t("toast.restoredN", { n: good.length }), timeout: 5000 });
      missionEvent({ type: "undo" });
    }
    if (bad.length) {
      pushToast({ kind: "help", title: t("toast.restoreFailed"), text: bad.map((r) => `${r.name}: ${r.message}`).join(" · "), timeout: 0 });
    }
    setTrashKey((k) => k + 1);
  }, [pushToast, missionEvent]);

  // ── Tabs ──
  const newTab = useCallback(() => {
    const cur = st.current.tabs.find((t) => t.id === st.current.active);
    const id = nextSid++;
    setTabs((all) => [...all, { id, title: cur?.title ?? "~", cwd: cur?.cwd ?? null, busy: null }]);
    setActive(id);
    missionEvent({ type: "tab", action: "new" });
  }, [missionEvent]);

  const selectTab = useCallback((id: number) => {
    if (id === st.current.active || !st.current.tabs.some((t) => t.id === id)) return;
    // Solange ein Dialog offen ist, nicht wegwechseln.
    if (st.current.ui[st.current.active]?.pending) return;
    setActive(id);
    missionEvent({ type: "tab", action: "switch" });
  }, [missionEvent]);

  const closeTab = useCallback((id: number) => {
    const all = st.current.tabs;
    if (all.length === 1) {
      // Letzter Tab: statt Leere eine frische Sitzung.
      const fresh = nextSid++;
      setTabs([{ id: fresh, title: "~", cwd: null, busy: null }]);
      setActive(fresh);
    } else {
      const idx = all.findIndex((t) => t.id === id);
      const rest = all.filter((t) => t.id !== id);
      setTabs(rest);
      if (id === st.current.active) setActive(rest[Math.max(0, idx - 1)].id);
    }
    setUi((u) => { const c = { ...u }; delete c[id]; return c; });
    missionEvent({ type: "tab", action: "close" });
  }, [missionEvent]);

  const cycleTab = useCallback((dir: 1 | -1) => {
    const all = st.current.tabs;
    const idx = all.findIndex((t) => t.id === st.current.active);
    selectTab(all[(idx + dir + all.length) % all.length].id);
  }, [selectTab]);

  const changeFont = useCallback((px: number) => {
    const n = Math.min(28, Math.max(10, px));
    setFontSize(n);
    try { localStorage.setItem("kompass.font", String(n)); } catch { /* egal */ }
    handles.current.forEach((h) => h.setFontSize(n));
  }, []);

  const togglePanel = useCallback((p: Panel) => {
    setPanel((cur) => (cur === p ? null : p));
    focusActive();
  }, [focusActive]);

  // ── Menü & Tastenkürzel ──
  const onMenu = useCallback((id: string) => {
    if (id === "tab-new") newTab();
    else if (id === "tab-close") closeTab(st.current.active);
    else if (id === "tab-next") cycleTab(1);
    else if (id === "tab-prev") cycleTab(-1);
    else if (id.startsWith("tab-")) {
      const t = st.current.tabs[Number(id.slice(4)) - 1];
      if (t) selectTab(t.id);
    } else if (id === "clear") handles.current.get(st.current.active)?.clear();
    else if (id === "panel-missions") togglePanel("missions");
    else if (id === "panel-trash") togglePanel("trash");
    else if (id === "settings") togglePanel("settings");
    else if (id === "font-bigger") changeFont(st.current.fontSize + 1);
    else if (id === "font-smaller") changeFont(st.current.fontSize - 1);
    else if (id === "font-reset") changeFont(14);
  }, [newTab, closeTab, cycleTab, selectTab, togglePanel, changeFont]);

  useEffect(() => {
    if (isTauri()) {
      const off = listen<string>("menu", (e) => onMenu(e.payload));
      return () => { void off.then((f) => f()); };
    }
    // Im normalen Browser (Vorschau) gibt es kein Mac-Menü → Kürzel selbst abfangen.
    const onKey = (e: KeyboardEvent) => {
      if (!e.metaKey) return;
      const map: Record<string, string> = { t: "tab-new", w: "tab-close", k: "clear", ",": "settings", "=": "font-bigger", "-": "font-smaller", "0": "font-reset" };
      let id = map[e.key.toLowerCase()];
      if (e.shiftKey && e.key === "]") id = "tab-next";
      if (e.shiftKey && e.key === "[") id = "tab-prev";
      if (/^[1-9]$/.test(e.key)) id = `tab-${e.key}`;
      if (id) { e.preventDefault(); onMenu(id); }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [onMenu]);

  // Esc bricht eine laufende KI-Anfrage ab.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const th = st.current.ui[st.current.active]?.thinking;
      if (e.key === "Escape" && th) {
        e.preventDefault();
        void answerGuard(st.current.active, th.id, "cancel");
        patchUi(st.current.active, { thinking: null });
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [patchUi]);

  // ── Nachrichten aus den Shells ──
  const onMessage = useCallback((sid: number, m: ServerMsg) => {
    const isActive = sid === st.current.active;
    const { t } = tRef.current;
    switch (m.kind) {
      case "verdict":
        patchUi(sid, { last: { level: m.verdict.level, command: m.command }, running: true, pending: m.needsConfirm ? { id: m.id, command: m.command, verdict: m.verdict } : null });
        patchTab(sid, { busy: m.needsConfirm ? null : m.command.trim().split(/\s+/)[0] });
        if (!m.needsConfirm && m.verdict.level === "care" && isActive) {
          const f = m.verdict.findings.filter((x) => x.level === "care");
          pushToast({ kind: "care", eyebrow: `${t("toast.care")} · ${m.command}`, title: f[0]?.title ?? t("toast.care"), text: f.map((x) => x.text).join(" ") });
        }
        break;
      case "live":
        patchUi(sid, (u) => ({ live: { line: m.line, explanation: m.explanation, nl: m.nl }, running: m.line ? u.running : false }));
        if (!m.line) patchTab(sid, { busy: null });
        break;
      case "thinking":
        patchUi(sid, { thinking: { id: m.id, query: m.query } });
        break;
      case "translated":
        patchUi(sid, { thinking: null });
        pushToast({
          kind: "nl",
          eyebrow: `${m.suggestion.source === "ki" ? t("toast.nlKi") : t("toast.nlLocal")} · ${m.query}`,
          title: m.suggestion.command,
          code: true,
          text: `${m.suggestion.text}. ${m.suggestion.needsInput ? t("toast.nlNeedsInput") : t("toast.nlRun")}`,
          timeout: 9000,
        });
        missionEvent({ type: "translated", query: m.query, command: m.suggestion.command });
        break;
      case "notUnderstood":
        patchUi(sid, { thinking: null });
        pushToast({
          kind: "help",
          eyebrow: `${t("toast.nlLocal")} · ${m.query}`,
          title: t("toast.notUnderstood"),
          text: m.kiAvailable ? m.reason : `${m.reason} ${t("toast.notUnderstoodHelp")}`,
          timeout: 15000,
          actions: m.kiAvailable ? undefined : [{ label: t("toast.setupKi"), onClick: () => setPanel("settings") }],
        });
        break;
      case "done":
        patchUi(sid, { running: false });
        patchTab(sid, { cwd: m.cwd, title: titleFor(m.cwd, st.current.home), busy: null });
        missionEvent({ type: "done", command: m.command, exitCode: m.exitCode, cwd: m.cwd });
        countCommand();
        // Spenden-Erinnerung: nie gleich zu Beginn, höchstens einmal pro Start, nie mitten in einem Dialog.
        if (!donateAsked.current && !st.current.ui[st.current.active]?.pending && shouldAsk(completedMissions(st.current.progress))) {
          donateAsked.current = true;
          setTimeout(() => {
            // Ist inzwischen ein Dialog offen? Dann beim nächsten Befehl erneut versuchen.
            if (st.current.ui[st.current.active]?.pending) donateAsked.current = false;
            else setDonateOpen(true);
          }, 1500);
        }
        break;
      case "help":
        if (!isActive) break;
        pushToast({
          kind: "help",
          eyebrow: `${t("toast.whatHappened")} · ${m.command}`,
          title: m.help.title,
          text: m.help.text,
          timeout: 20000,
          actions: m.help.suggestion ? [{ label: t("toast.insert", { cmd: m.help.suggestion }), onClick: () => insert(m.help.suggestion!), primary: true }] : undefined,
        });
        break;
      case "trashed": {
        const names = m.items.map((i) => i.name);
        pushToast({
          kind: "trashed",
          title: m.items.length === 1 ? t("toast.trashed1", { name: names[0] }) : t("toast.trashedN", { n: m.items.length }),
          text: m.items.length > 1 ? names.slice(0, 4).join(", ") + (m.items.length > 4 ? " …" : "") : t("toast.trashedText"),
          timeout: 15000,
          actions: [{ label: t("toast.undo"), onClick: () => void restore(m.batch), primary: true }],
        });
        setTrashKey((k) => k + 1);
        break;
      }
      case "exit":
        patchUi(sid, { exited: true, running: false });
        break;
    }
  }, [patchUi, patchTab, pushToast, missionEvent, insert, restore]);

  const choose = useCallback((choice: Choice) => {
    const sid = st.current.active;
    const p = st.current.ui[sid]?.pending;
    if (p) void answerGuard(sid, p.id, choice);
    patchUi(sid, { pending: null });
    focusActive();
  }, [patchUi, focusActive]);

  const register = useCallback((sid: number, h: TermHandle | null) => {
    if (h) handles.current.set(sid, h);
    else handles.current.delete(sid);
  }, []);

  const cur = tabUi(active);
  const activeTab = tabs.find((t) => t.id === active);

  return (
    <div className={`app ${panel ? "with-panel" : ""}`}>
      <header className="titlebar" data-tauri-drag-region>
        <div className="brand" data-tauri-drag-region>
          <span className="brand-mark">◆</span> Kompass
        </div>
        <div className="title-actions">
          <div className="guard-pill" data-level={cur.last?.level ?? "idle"} title={t("title.guardHint")}>
            <span className="dot" />
            {cur.last ? (
              <>
                <span className="pill-label">{t(`level.${cur.last.level}` as const)}</span>
                <code className="pill-cmd">{cur.last.command}</code>
              </>
            ) : (
              <span className="pill-label">{t("title.guardActive")}</span>
            )}
          </div>
          <button className={`tab ${panel === "missions" ? "active" : ""}`} onClick={() => togglePanel("missions")}>
            🧭 {t("panel.missions")} <span className="tab-badge">{progress.size}/{TOTAL_STEPS}</span>
          </button>
          <button className={`tab ${panel === "trash" ? "active" : ""}`} onClick={() => togglePanel("trash")}>🗑 {t("panel.trash")}</button>
          <div className="lang-switch" role="group" aria-label={t("settings.language")}>
            {LANGS.map((x) => (
              <button key={x.id} className={x.id === lang ? "active" : ""} onClick={() => setLang(x.id)} title={x.name} aria-pressed={x.id === lang}>{x.label}</button>
            ))}
          </div>
          <button className={`tab icon ${panel === "settings" ? "active" : ""}`} onClick={() => togglePanel("settings")} title={t("title.settings")} aria-label={t("panel.settings")}>⚙︎</button>
        </div>
      </header>

      <main className="stage">
        <div className="terminal-col">
          <TabBar tabs={tabs} active={active} onSelect={selectTab} onClose={closeTab} onNew={newTab} />
          <div className="terminal-card">
            {tabs.map((t) => (
              <TerminalView
                key={t.id}
                sid={t.id}
                active={t.id === active}
                startCwd={t.cwd}
                fontSize={fontSize}
                onMessage={onMessage}
                register={register}
              />
            ))}
            {cur.exited && (
              <div className="exited">
                <p>{t("exited.text")}</p>
                <button className="btn btn-primary" onClick={() => closeTab(active)}>{t("exited.button")}</button>
              </div>
            )}
          </div>
          <ExplainBar line={cur.live.line} explanation={cur.live.explanation} nl={cur.live.nl} running={cur.running} thinking={cur.thinking?.query ?? null} />
        </div>

        {panel && (
          <aside className="side-panel">
            <div className="panel-title">
              <h3>{t(`panel.${panel}` as const)}</h3>
              <button className="toast-close" onClick={() => togglePanel(panel)} aria-label={t("common.close")}>×</button>
            </div>
            {panel === "missions" && (
              <MissionsPanel
                done={progress}
                chosen={chosen}
                onChoose={chooseMission}
                onInsert={insert}
                onReset={() => { const empty = new Set<string>(); setProgress(empty); saveProgress(empty); setChosen(null); saveChosen(null); }}
              />
            )}
            {panel === "trash" && <TrashPanel refreshKey={trashKey} home={home} onRestore={(b) => void restore(b)} />}
            {panel === "settings" && <SettingsPanel fontSize={fontSize} onFontSize={changeFont} onInsert={insert} onDonate={() => setDonateOpen(true)} />}
          </aside>
        )}
      </main>

      <Toasts items={toasts} onClose={closeToast} />
      {cur.pending && <ConfirmDialog command={cur.pending.command} verdict={cur.pending.verdict} onChoose={choose} />}
      {donateOpen && !cur.pending && <DonateDialog onClose={() => { setDonateOpen(false); focusActive(); }} />}
      <span className="sr-only" aria-live="polite">{activeTab ? shortPath(activeTab.cwd ?? "~", home) : ""}</span>
    </div>
  );
}
