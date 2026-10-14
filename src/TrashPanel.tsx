import { useEffect, useState } from "react";
import { TrashBatch, trashList } from "./shell";
import { T, useI18n } from "./i18n";

type Props = { refreshKey: number; home: string; onRestore: (batch: number) => void };

function when(ms: number, t: T, locale: string) {
  const d = new Date(ms);
  const today = new Date();
  const yesterday = new Date(Date.now() - 864e5);
  const time = d.toLocaleTimeString(locale, { hour: "2-digit", minute: "2-digit" });
  if (d.toDateString() === today.toDateString()) return t("trash.today", { time });
  if (d.toDateString() === yesterday.toDateString()) return t("trash.yesterday", { time });
  return `${d.toLocaleDateString(locale, { day: "numeric", month: "short" })}, ${time}`;
}

export default function TrashPanel({ refreshKey, home, onRestore }: Props) {
  const { t, lang } = useI18n();
  const locale = lang === "en" ? "en-US" : "de-DE";
  const [batches, setBatches] = useState<TrashBatch[] | null>(null);
  useEffect(() => {
    trashList().then(setBatches).catch(() => setBatches([]));
  }, [refreshKey]);

  const short = (p: string) => (home && p.startsWith(home) ? "~" + p.slice(home.length) : p);

  if (batches === null) return <div className="panel-body muted">{t("trash.loading")}</div>;
  if (batches.length === 0)
    return (
      <div className="panel-body empty">
        <div className="empty-icon">🗑</div>
        <p><strong>{t("trash.emptyTitle")}</strong></p>
        <p className="muted">{t("trash.emptyText")}</p>
      </div>
    );

  return (
    <div className="panel-body">
      {batches.map((b) => {
        const restorable = b.items.some((i) => i.available);
        return (
          <section key={b.batch} className="trash-batch">
            <div className="trash-head">
              <span>{when(b.batch, t, locale)}</span>
              {restorable ? (
                <button className="btn btn-sm" onClick={() => onRestore(b.batch)}>{t("trash.restore")}</button>
              ) : (
                <span className="muted small">{t("trash.gone")}</span>
              )}
            </div>
            <ul>
              {b.items.map((i, n) => (
                <li key={n} className={i.available ? "" : "gone"}>
                  <span>{i.isDir ? "📁" : "📄"}</span>
                  <div>
                    <p className="t-name">{i.name}</p>
                    <p className="muted small">{t("trash.from", { folder: short(i.folder) })}</p>
                  </div>
                </li>
              ))}
            </ul>
          </section>
        );
      })}
    </div>
  );
}
