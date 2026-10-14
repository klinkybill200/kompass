import { useEffect, useRef, useState } from "react";
import { Level, MISSIONS, TOTAL_STEPS, currentMission, key, nextStep } from "./missions";
import { useI18n } from "./i18n";

type Props = {
  done: Set<string>;
  chosen: string | null;
  onChoose: (id: string) => void;
  onInsert: (cmd: string) => void;
  onReset: () => void;
};

const LEVELS: Level[] = ["Einstieg", "Fortgeschritten", "Profi"];

export default function MissionsPanel({ done, chosen, onChoose, onInsert, onReset }: Props) {
  const { t, l } = useI18n();
  const active = currentMission(chosen, done);
  const [open, setOpen] = useState<string | null>(null);
  const [confirmReset, setConfirmReset] = useState(false);
  const shown = open ?? active?.id ?? null;
  const pct = Math.round((done.size / TOTAL_STEPS) * 100);
  const playingRef = useRef<HTMLElement>(null);

  // Die laufende Mission in den sichtbaren Bereich holen.
  useEffect(() => {
    playingRef.current?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }, [active?.id]);

  return (
    <div className="panel-body">
      <div className="progress-card">
        <div className="progress-top">
          <strong>{done.size === TOTAL_STEPS ? t("missions.allDone") : t("missions.progress")}</strong>
          <span>{done.size} / {TOTAL_STEPS}</span>
        </div>
        <div className="progress"><div style={{ width: `${pct}%` }} /></div>
      </div>

      {LEVELS.map((level) => (
        <div key={level} className="mission-group">
          <p className="group-label">{t(`missions.level.${level}` as const)}</p>
          {MISSIONS.filter((m) => m.level === level).map((m) => {
            const count = m.steps.filter((s) => done.has(key(m, s))).length;
            const complete = count === m.steps.length;
            const current = nextStep(m, done);
            const expanded = shown === m.id;
            const playing = active?.id === m.id;
            return (
              <section key={m.id} ref={playing ? playingRef : undefined} className={`mission ${complete ? "complete" : ""} ${expanded ? "expanded" : ""} ${playing ? "playing" : ""}`}>
                <button className="mission-head" onClick={() => setOpen(expanded ? "" : m.id)} aria-expanded={expanded}>
                  <span className="mission-emoji">{m.emoji}</span>
                  <span className="mission-title">{l(m.title)}</span>
                  {playing && <span className="playing-badge">{t("missions.playing")}</span>}
                  <span className="mission-count">{complete ? "✓" : `${count}/${m.steps.length}`}</span>
                </button>
                {expanded && (
                  <div className="mission-content">
                    <p className="mission-intro">{l(m.intro)}</p>
                    {!playing && !complete && (
                      <button className="btn btn-sm btn-primary play" onClick={() => onChoose(m.id)}>{t("missions.play")}</button>
                    )}
                    <ol className="steps">
                      {m.steps.map((s) => {
                        const isDone = done.has(key(m, s));
                        const isCurrent = s === current && playing;
                        return (
                          <li key={s.id} className={isDone ? "done" : isCurrent ? "current" : "todo"}>
                            <span className="step-mark">{isDone ? "✓" : ""}</span>
                            <div>
                              <p className="step-title">{l(s.title)}</p>
                              {isCurrent && (
                                <>
                                  <p className="step-text">{l(s.text)}</p>
                                  {s.hint && (
                                    <button className="hint" onClick={() => onInsert(l(s.hint!))} title={t("missions.insertTitle")}>
                                      <code>{l(s.hint)}</code><span>{t("missions.insert")}</span>
                                    </button>
                                  )}
                                  {s.keys && <p className="keys"><kbd>{s.keys}</kbd></p>}
                                </>
                              )}
                            </div>
                          </li>
                        );
                      })}
                    </ol>
                  </div>
                )}
              </section>
            );
          })}
        </div>
      ))}

      {done.size > 0 && (
        <button
          className="btn-link reset"
          onClick={() => { if (confirmReset) { onReset(); setConfirmReset(false); } else setConfirmReset(true); }}
          onBlur={() => setConfirmReset(false)}
        >
          {confirmReset ? t("missions.resetConfirm") : t("missions.reset")}
        </button>
      )}
    </div>
  );
}
