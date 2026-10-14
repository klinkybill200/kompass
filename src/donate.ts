/**
 * Spenden-Erinnerung.
 * Kompass ist Freeware. Das Spendenfenster erscheint NICHT gleich zu Beginn, sondern erst,
 * wenn jemand Kompass wirklich nutzt – und nach einer Spende nie wieder.
 */

export const DONATION_AMOUNTS = [5, 10, 20] as const;
export type Amount = (typeof DONATION_AMOUNTS)[number];

type State = {
  firstSeen: number;   // erster Start (ms)
  commands: number;    // ausgeführte Befehle insgesamt
  snoozeUntil: number; // „Vielleicht später“ → bis dahin Ruhe
  donated: boolean;    // nach „Ja, gespendet“ nie mehr fragen
};

const KEY = "kompass.donate.v1";
const DAY = 24 * 60 * 60 * 1000;

/** Ab wann gefragt wird: nach 3 Tagen und 30 Befehlen – oder nach 3 geschafften Missionen (mind. 10 Befehle). */
const MIN_DAYS = 3;
const MIN_COMMANDS = 30;
const MIN_MISSIONS = 3;
const SNOOZE_DAYS = 14;

function load(): State {
  try {
    const s = JSON.parse(localStorage.getItem(KEY) ?? "null");
    if (s && typeof s.firstSeen === "number") return s;
  } catch {
    /* egal */
  }
  const fresh: State = { firstSeen: Date.now(), commands: 0, snoozeUntil: 0, donated: false };
  save(fresh);
  return fresh;
}

function save(s: State) {
  try {
    localStorage.setItem(KEY, JSON.stringify(s));
  } catch {
    /* Speichern ist nur Komfort */
  }
}

export function countCommand() {
  const s = load();
  s.commands += 1;
  save(s);
}

export function shouldAsk(completedMissions: number): boolean {
  const s = load();
  if (s.donated || Date.now() < s.snoozeUntil) return false;
  const days = (Date.now() - s.firstSeen) / DAY;
  return (days >= MIN_DAYS && s.commands >= MIN_COMMANDS) || (completedMissions >= MIN_MISSIONS && s.commands >= 10);
}

export function snooze(days = SNOOZE_DAYS) {
  const s = load();
  s.snoozeUntil = Date.now() + days * DAY;
  save(s);
}

export function markDonated() {
  const s = load();
  s.donated = true;
  save(s);
}

export const hasDonated = () => load().donated;
