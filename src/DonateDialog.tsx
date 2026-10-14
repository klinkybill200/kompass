import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "./i18n";
import { DONATION_AMOUNTS, Amount, markDonated, snooze } from "./donate";

type Props = { onClose: () => void };
type Stage = "ask" | "wait" | "thanks";

export default function DonateDialog({ onClose }: Props) {
  const { t } = useI18n();
  const [stage, setStage] = useState<Stage>("ask");
  const [amount, setAmount] = useState<Amount>(10);
  const first = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    first.current?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") { e.preventDefault(); later(); }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [stage]);

  const later = () => {
    if (stage !== "thanks") snooze();
    onClose();
  };

  const donate = (a: Amount) => {
    setAmount(a);
    if ("__TAURI_INTERNALS__" in window) void invoke("open_donation", { amount: a });
    setStage("wait");
  };

  return (
    <div className="overlay" role="dialog" aria-modal="true" aria-labelledby="donate-title">
      <div className="dialog donate">
        {stage === "ask" && (
          <>
            <div className="donate-hero" aria-hidden>
              <span className="donate-mark">◆</span>
            </div>
            <p className="eyebrow center">{t("donate.eyebrow")}</p>
            <h2 id="donate-title" className="center">{t("donate.title")}</h2>
            <p className="donate-text">{t("donate.text")}</p>
            <div className="amounts">
              {DONATION_AMOUNTS.map((a, i) => (
                <button key={a} ref={i === 1 ? first : undefined} className={`amount ${a === 10 ? "featured" : ""}`} onClick={() => donate(a)}>
                  <span className="amount-value">${a}</span>
                </button>
              ))}
            </div>
            <p className="donate-via">{t("donate.via")}</p>
            <div className="actions center">
              <button className="btn-link" onClick={later}>{t("donate.later")}</button>
            </div>
          </>
        )}

        {stage === "wait" && (
          <>
            <h2 id="donate-title" className="center">{t("donate.waitTitle")}</h2>
            <p className="donate-text">{t("donate.waitText")}</p>
            <p className="donate-amount center">${amount}</p>
            <div className="actions center">
              <button className="btn" onClick={() => { snooze(3); onClose(); }}>{t("donate.notYet")}</button>
              <button ref={first} className="btn btn-primary" onClick={() => { markDonated(); setStage("thanks"); }}>{t("donate.done")}</button>
            </div>
          </>
        )}

        {stage === "thanks" && (
          <>
            <div className="donate-hero thanks" aria-hidden>💚</div>
            <h2 id="donate-title" className="center">{t("donate.thanksTitle")}</h2>
            <p className="donate-text">{t("donate.thanksText")}</p>
            <div className="actions center">
              <button ref={first} className="btn btn-primary" onClick={onClose}>{t("donate.ok")}</button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
