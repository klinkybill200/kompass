import { useEffect } from "react";
import { useI18n } from "./i18n";

export type ToastAction = { label: string; onClick: () => void; primary?: boolean };
export type ToastItem = {
  id: number;
  kind: "care" | "help" | "trashed" | "success" | "mission" | "nl";
  eyebrow?: string;
  title: string;
  text?: string;
  actions?: ToastAction[];
  timeout?: number;
  /** Titel als Befehl (Schreibmaschinenschrift) anzeigen */
  code?: boolean;
};

const ICON: Record<ToastItem["kind"], string> = { care: "!", help: "?", trashed: "🗑", success: "✓", mission: "🏆", nl: "→" };

function Toast({ t, onClose }: { t: ToastItem; onClose: (id: number) => void }) {
  const { t: tr } = useI18n();
  useEffect(() => {
    const ms = t.timeout ?? 10000;
    if (ms <= 0) return;
    const timer = setTimeout(() => onClose(t.id), ms);
    return () => clearTimeout(timer);
  }, [t.id, t.timeout, onClose]);

  return (
    <div className={`toast ${t.kind}`} role="status">
      {t.kind === "mission" && <div className="confetti" aria-hidden>{Array.from("🎉✨🎊⭐🎉✨").map((c, i) => <span key={i} style={{ ["--i" as string]: i }}>{c}</span>)}</div>}
      <span className={`level-icon ${t.kind}`} aria-hidden>{ICON[t.kind]}</span>
      <div className="toast-body">
        {t.eyebrow && <p className="eyebrow">{t.eyebrow}</p>}
        <p className="toast-title">{t.code ? <code className="toast-code">{t.title}</code> : t.title}</p>
        {t.text && <p className="toast-text">{t.text}</p>}
        {t.actions && (
          <div className="toast-actions">
            {t.actions.map((a, i) => (
              <button key={i} className={`btn btn-sm ${a.primary ? "btn-primary" : ""}`} onClick={() => { a.onClick(); onClose(t.id); }}>
                {a.label}
              </button>
            ))}
          </div>
        )}
      </div>
      <button className="toast-close" onClick={() => onClose(t.id)} aria-label={tr("common.close")}>×</button>
    </div>
  );
}

export default function Toasts({ items, onClose }: { items: ToastItem[]; onClose: (id: number) => void }) {
  return (
    <div className="toasts">
      {items.map((t) => <Toast key={t.id} t={t} onClose={onClose} />)}
    </div>
  );
}
