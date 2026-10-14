import { useEffect, useState } from "react";
import { Explanation, NlLive } from "./shell";
import { TIP_COUNT, TKey, useI18n } from "./i18n";

type Props = {
  line: string;
  explanation: Explanation | null;
  nl: NlLive | null;
  running: boolean;
  thinking: string | null;
};

export default function ExplainBar({ line, explanation, nl, running, thinking }: Props) {
  const { t } = useI18n();
  const [tip, setTip] = useState(0);
  useEffect(() => {
    const id = setInterval(() => setTip((i) => (i + 1) % TIP_COUNT), 9000);
    return () => clearInterval(id);
  }, []);

  if (thinking) {
    return (
      <div className="explain-bar nl" data-level="safe">
        <div className="explain-head">
          <span className="explain-badge ki">{t("explain.ki")}</span>
          <p className="explain-summary"><span className="spinner" aria-hidden /> {t("explain.kiAsking", { q: thinking })}</p>
          <span className="explain-level">{t("explain.escCancels")}</span>
        </div>
      </div>
    );
  }

  if (line.trim() && nl) {
    const s = nl.suggestion;
    return (
      <div className="explain-bar nl" data-level="safe">
        <div className="explain-head">
          <span className="explain-badge nl">{t("explain.nl")}</span>
          <p className="explain-summary">{s ? s.text : nl.kiAvailable ? t("explain.kiOnEnter") : t("explain.notKnown")}</p>
          <span className="explain-level">{s || nl.kiAvailable ? t("explain.enterTranslates") : ""}</span>
        </div>
        {s && (
          <div className="chips">
            <div className="chip chip-cmd chip-wide">
              <span>{t("explain.becomes")}</span>
              <code>{s.command}</code>
            </div>
            {s.needsInput && <div className="chip chip-unknown"><span>{t("explain.fillIn")}</span></div>}
          </div>
        )}
      </div>
    );
  }

  if (!line.trim() || !explanation) {
    return (
      <div className="explain-bar idle">
        <span className="explain-badge">{t("explain.tip")}</span>
        <p key={tip} className="explain-tip">{t(`tips.${tip}` as TKey)}</p>
      </div>
    );
  }

  return (
    <div className="explain-bar" data-level={explanation.level}>
      <div className="explain-head">
        <span className="explain-badge">{running ? t("explain.running") : t("explain.happens")}</span>
        <p className="explain-summary">{explanation.summary}</p>
        <span className="explain-level"><span className="dot" />{t(`levelLower.${explanation.level}` as const)}</span>
      </div>
      <div className="chips">
        {explanation.parts.map((p, i) => (
          <div key={i} className={`chip chip-${p.kind}`} title={p.meaning}>
            <code>{p.token}</code>
            <span>{p.meaning}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
