import { useEffect, useRef } from "react";
import { Choice, Target, Verdict } from "./shell";
import { T, useI18n } from "./i18n";

type Props = {
  command: string;
  verdict: Verdict;
  onChoose: (c: Choice) => void;
};

function targetLabel(x: Target, t: T) {
  if (x.kind === "dir") return x.entries === null ? t("dialog.folder") : `${t("dialog.folder")} · ${x.entries} ${x.entries === 1 ? t("dialog.entries1") : t("dialog.entriesN")}`;
  if (x.kind === "file") return t("dialog.file");
  if (x.kind === "missing") return t("dialog.missing");
  return t("dialog.unknown");
}

export default function ConfirmDialog({ command, verdict, onChoose }: Props) {
  const primary = useRef<HTMLButtonElement>(null);
  const trash = verdict.trash;
  const danger = verdict.findings.filter((f) => f.level === "danger");
  const others = verdict.findings.filter((f) => f.level !== "danger");
  const { t } = useI18n();
  const title = trash ? t("dialog.trashTitle") : danger[0]?.title ?? t("dialog.sure");

  useEffect(() => {
    primary.current?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") { e.preventDefault(); onChoose("cancel"); }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onChoose]);

  const shown = trash?.targets.slice(0, 8) ?? [];
  const more = (trash?.targets.length ?? 0) - shown.length;

  return (
    <div className="overlay" role="dialog" aria-modal="true" aria-labelledby="dlg-title">
      <div className="dialog">
        <div className="dialog-head">
          <span className={`level-icon ${trash ? "trash" : "danger"}`} aria-hidden>{trash ? "🗑" : "!"}</span>
          <div>
            <p className="eyebrow">{t("dialog.paused")}</p>
            <h2 id="dlg-title">{title}</h2>
          </div>
        </div>

        <code className="cmd">{command}</code>

        <ul className="findings">
          {[...danger, ...others].map((f, i) => (
            <li key={i} className={`finding ${f.level}`}>
              <strong>{f.title}</strong>
              <span>{f.text}</span>
            </li>
          ))}
        </ul>

        {shown.length > 0 && (
          <ul className="targets">
            {shown.map((x, i) => (
              <li key={i} className={x.kind}>
                <span className="t-icon">{x.kind === "dir" ? "📁" : x.kind === "missing" ? "∅" : "📄"}</span>
                <span className="t-name">{x.name}</span>
                <span className="t-meta">{targetLabel(x, t)}</span>
              </li>
            ))}
            {more > 0 && <li className="t-more">{t("dialog.more", { n: more })}</li>}
          </ul>
        )}

        <div className="actions">
          {trash ? (
            <>
              <button className="btn btn-link" onClick={() => onChoose("run")}>{t("dialog.deleteForever")}</button>
              <span className="spacer" />
              <button className="btn" onClick={() => onChoose("cancel")}>{t("dialog.cancel")}</button>
              <button ref={primary} className="btn btn-primary" onClick={() => onChoose("trash")}>{t("dialog.toTrash")}</button>
            </>
          ) : (
            <>
              <span className="spacer" />
              <button ref={primary} className="btn" onClick={() => onChoose("cancel")}>{t("dialog.ratherNot")}</button>
              <button className="btn btn-danger" onClick={() => onChoose("run")}>{t("dialog.run")}</button>
            </>
          )}
        </div>
        <p className="hint">{t("dialog.keys")}</p>
      </div>
    </div>
  );
}
