import { useEffect, useState } from "react";
import { KiStatus, Suggestion, kiClearKey, kiSetKey, kiStatus, kiTest } from "./shell";
import { LANGS, useI18n } from "./i18n";
import { hasDonated } from "./donate";

type Props = { fontSize: number; onFontSize: (px: number) => void; onInsert: (cmd: string) => void; onDonate: () => void };

const EXAMPLES = {
  de: [
    "zeige alle dateien mit details",
    "finde alle pdf dateien auf dem schreibtisch",
    "wie viel speicherplatz ist frei",
    "welche programme laufen gerade",
    "erstelle einen ordner namens Projekte",
    "packe den ordner Fotos",
    "die 10 größten dateien",
    "was ist meine ip adresse",
  ],
  en: [
    "show all files with details",
    "find all pdf files on the desktop",
    "how much disk space is free",
    "which programs are running",
    "create a folder called Projects",
    "compress the folder Photos",
    "the largest files",
    "what is my ip address",
  ],
};

export default function SettingsPanel({ fontSize, onFontSize, onInsert, onDonate }: Props) {
  const { t, lang, setLang } = useI18n();
  const [status, setStatus] = useState<KiStatus | null>(null);
  const [key, setKey] = useState("");
  const [msg, setMsg] = useState<{ ok: boolean; text: string } | null>(null);
  const [busy, setBusy] = useState(false);
  const [test, setTest] = useState<Suggestion | null>(null);
  const donated = hasDonated();

  useEffect(() => {
    kiStatus().then(setStatus).catch(() => setStatus(null));
  }, []);

  const save = async () => {
    setMsg(null);
    try {
      setStatus(await kiSetKey(key));
      setKey("");
      setMsg({ ok: true, text: t("settings.saved") });
    } catch (e) {
      setMsg({ ok: false, text: String(e) });
    }
  };

  const runTest = async () => {
    setBusy(true);
    setMsg(null);
    setTest(null);
    try {
      setTest(await kiTest());
    } catch (e) {
      setMsg({ ok: false, text: String(e) });
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="panel-body settings">
      <section className="setting-card">
        <h4>{t("settings.language")}</h4>
        <div className="segmented" role="radiogroup" aria-label={t("settings.language")}>
          {LANGS.map((x) => (
            <button key={x.id} role="radio" aria-checked={x.id === lang} className={x.id === lang ? "active" : ""} onClick={() => setLang(x.id)}>
              {x.name}
            </button>
          ))}
        </div>
        <p className="muted small">{t("settings.languageHint")}</p>
      </section>

      <section className="setting-card">
        <h4>{t("settings.nl")}</h4>
        <p className="muted small">{t("settings.nlText")}</p>
        <div className="examples">
          {EXAMPLES[lang].map((ex) => (
            <button key={ex} className="example" onClick={() => onInsert(ex)}>{ex}</button>
          ))}
        </div>
      </section>

      <section className="setting-card">
        <div className="setting-head">
          <h4>{t("settings.ki")}</h4>
          <span className={`status-pill ${status?.configured ? "on" : ""}`}>
            {status?.configured ? (status.source === "env" ? t("settings.kiOnEnv") : t("settings.kiOn")) : t("settings.kiOff")}
          </span>
        </div>
        <p className="muted small">{t("settings.kiText")}</p>
        {status?.source !== "env" && (
          <div className="key-row">
            <input
              type="password"
              placeholder={status?.configured ? t("settings.kiNewKey") : "sk-ant-…"}
              value={key}
              onChange={(e) => setKey(e.target.value)}
              autoComplete="off"
              spellCheck={false}
            />
            <button className="btn btn-sm btn-primary" disabled={!key.trim()} onClick={save}>{t("settings.save")}</button>
          </div>
        )}
        {status?.configured && (
          <div className="key-actions">
            <button className="btn btn-sm" disabled={busy} onClick={runTest}>{busy ? t("settings.testing") : t("settings.test")}</button>
            {status.source === "keychain" && (
              <button className="btn-link" onClick={async () => { setStatus(await kiClearKey()); setTest(null); setMsg({ ok: true, text: t("settings.removed") }); }}>
                {t("settings.removeKey")}
              </button>
            )}
          </div>
        )}
        {test && (
          <p className="test-result">
            ✓ {t("settings.testResult")} <code>{test.command}</code>
          </p>
        )}
        {msg && <p className={`form-msg ${msg.ok ? "ok" : "err"}`}>{msg.text}</p>}
        <p className="muted small">{t("settings.model", { model: status?.model ?? "–" })}</p>
      </section>

      <section className="setting-card">
        <h4>{t("settings.font")}</h4>
        <div className="font-row">
          <button className="btn btn-sm" onClick={() => onFontSize(fontSize - 1)} aria-label={t("settings.smaller")}>A−</button>
          <span>{fontSize} px</span>
          <button className="btn btn-sm" onClick={() => onFontSize(fontSize + 1)} aria-label={t("settings.bigger")}>A+</button>
          <span className="muted small">⌘+ / ⌘− / ⌘0</span>
        </div>
      </section>

      <section className="setting-card">
        <h4>{t("settings.shortcuts")}</h4>
        <ul className="shortcuts">
          <li><kbd>⌘T</kbd> {t("settings.sc.newTab")}</li>
          <li><kbd>⌘W</kbd> {t("settings.sc.closeTab")}</li>
          <li><kbd>⌘1</kbd>…<kbd>⌘9</kbd> {t("settings.sc.jump")}</li>
          <li><kbd>⌘⇧[</kbd> <kbd>⌘⇧]</kbd> {t("settings.sc.prevNext")}</li>
          <li><kbd>⌘K</kbd> {t("settings.sc.clear")}</li>
          <li><kbd>⌘⇧M</kbd> {t("settings.sc.panels", { trash: "⌘⇧P", settings: "⌘," })}</li>
          <li><kbd>Tab</kbd> {t("settings.sc.basics", { up: "↑", ctrlc: "Ctrl+C" })}</li>
        </ul>
      </section>

      <section className="setting-card support">
        <h4>💚 {t("settings.support")}</h4>
        <p className="muted small">{donated ? t("settings.supportThanks") : t("settings.supportText")}</p>
        <div>
          <button className="btn btn-sm" onClick={onDonate}>{t("settings.supportButton")}</button>
        </div>
      </section>
    </div>
  );
}
