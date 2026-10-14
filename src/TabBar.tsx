import { useI18n } from "./i18n";

export type Tab = { id: number; title: string; cwd: string | null; busy: string | null };

type Props = {
  tabs: Tab[];
  active: number;
  onSelect: (id: number) => void;
  onClose: (id: number) => void;
  onNew: () => void;
};

export default function TabBar({ tabs, active, onSelect, onClose, onNew }: Props) {
  const { t } = useI18n();
  return (
    <div className="tabbar" role="tablist">
      {tabs.map((tb, i) => (
        <div
          key={tb.id}
          role="tab"
          aria-selected={tb.id === active}
          className={`term-tab ${tb.id === active ? "active" : ""}`}
          onClick={() => onSelect(tb.id)}
          onAuxClick={(e) => { if (e.button === 1) onClose(tb.id); }}
          title={tb.cwd ?? ""}
        >
          {tb.busy ? <span className="tab-busy" aria-hidden /> : <span className="tab-icon" aria-hidden>›_</span>}
          <span className="tab-title">{tb.busy ?? tb.title}</span>
          {i < 9 && <span className="tab-key">⌘{i + 1}</span>}
          {tabs.length > 1 && (
            <button className="tab-close" onClick={(e) => { e.stopPropagation(); onClose(tb.id); }} aria-label={t("tabs.close")}>×</button>
          )}
        </div>
      ))}
      <button className="tab-new" onClick={onNew} title={t("tabs.new")} aria-label={t("tabs.new")}>+</button>
    </div>
  );
}
