# ---------------------------------------------------------------------------
# Kompass-Shell-Integration
# Die Enter-Taste wird umgeleitet: Bevor eine Zeile ausgeführt wird, schickt
# sie dieses Skript unsichtbar an Kompass (OSC-Sequenz 7770) und wartet auf
# die Antwort:  r.          -> unverändert ausführen
#               r<base64>.  -> stattdessen diesen Befehl ausführen
#               e<base64>.  -> diesen Befehl in die Eingabezeile setzen (Klarsprache)
#               n<base64>.  -> Klarsprache nicht verstanden, Zeile bleibt stehen (+ Meldung)
#               c<base64>.  -> abbrechen (+ Meldung in der gewählten Sprache)
# ---------------------------------------------------------------------------
zmodload zsh/parameter

# Ist das erste Wort ein Befehl, den die zsh kennt? Ergebnis in REPLY (1/0).
# Wichtig für Klarsprache: „zeige alle dateien“ → „zeige“ ist kein Befehl.
_kompass_known() {
  local -a ws
  ws=(${(z)1})
  local f=${(Q)ws[1]}
  REPLY=1
  [[ -z "$f" || "$f" == *[/=]* ]] && return
  if [[ "$f" == \?* ]]; then REPLY=0; return; fi
  (( $+commands[$f] || $+aliases[$f] || $+functions[$f] || $+builtins[$f] || ${reswords[(Ie)$f]} )) || REPLY=0
}

kompass-accept-line() {
  emulate -L zsh
  local cmd="$BUFFER"
  # Leere Zeile: nichts zu prüfen.
  if [[ -z "${cmd//[[:space:]]/}" ]]; then
    zle .accept-line
    return
  fi

  local enc pwdenc reply='' c='' REPLY
  enc=$(print -rn -- "$cmd" | /usr/bin/base64)
  pwdenc=$(print -rn -- "$PWD" | /usr/bin/base64)
  _kompass_known "$cmd"
  print -rn -- $'\e]7770;'"${enc};${pwdenc};${REPLY}"$'\a'

  # Antwort Zeichen für Zeichen lesen, bis zum Punkt.
  while read -rsk 1 -t 900 c; do
    [[ "$c" == "." ]] && break
    reply+="$c"
  done

  # Mitgeschickte Meldung (Sprache wählt Kompass) – sonst deutscher Standardtext.
  local msg=''
  if [[ "${reply[1]}" == [cn] ]] && (( ${#reply} > 1 )); then
    msg=$(print -rn -- "${reply[2,-1]}" | /usr/bin/base64 -D)
  fi

  if [[ "${reply[1]}" == "e" ]]; then
    # Klarsprache übersetzt: Befehl hinschreiben, noch nicht ausführen.
    print -s -- "$cmd"
    BUFFER=$(print -rn -- "${reply[2,-1]}" | /usr/bin/base64 -D)
    CURSOR=${#BUFFER}
    return
  elif [[ "${reply[1]}" == "n" ]]; then
    # Nicht verstanden: Zeile stehen lassen, damit man sie umformulieren kann.
    zle -I
    print -r -- $'\e[33m'"${msg:-🤔  Das habe ich nicht verstanden – formulier es anders oder tippe einen Befehl.}"$'\e[0m'
    return
  elif [[ "${reply[1]}" == "r" ]]; then
    if (( ${#reply} > 1 )); then
      BUFFER=$(print -rn -- "${reply[2,-1]}" | /usr/bin/base64 -D)
      CURSOR=${#BUFFER}
    fi
    zle .accept-line
  else
    # Abgebrochen: Befehl landet im Verlauf (Pfeil-hoch holt ihn zurück).
    print -s -- "$cmd"
    BUFFER=''
    zle -I
    print -r -- $'\e[33m'"${msg:-↩  Abgebrochen – es ist nichts passiert.}"$'\e[0m'
  fi
}
zle -N accept-line kompass-accept-line

# Ersatz für rm: legt Dateien in den macOS-Papierkorb (Rust: --kompass-trash).
kompass-trash() {
  local -a files
  local a opts_done=0
  for a in "$@"; do
    if (( ! opts_done )) && [[ "$a" == "--" ]]; then opts_done=1; continue; fi
    if (( ! opts_done )) && [[ "$a" == -?* ]]; then continue; fi
    files+=("$a")
  done
  if (( ${#files} == 0 )); then
    print "kompass-trash: Es wurden keine Dateien angegeben."
    return 1
  fi
  "$KOMPASS_BIN" --kompass-trash -- "${files[@]}"
}

# Letzten Papierkorb-Vorgang rückgängig machen (Dateien zurück an den alten Ort).
kompass-undo() {
  "$KOMPASS_BIN" --kompass-undo
}
alias rueckgaengig=kompass-undo

# ---------------------------------------------------------------------------
# Live-Erklärung: meldet bei jeder Änderung die aktuelle Eingabe (OSC 7771).
# Format: <Ordner> 0x1F <Zeile>. Zeilen mit Steuerzeichen werden ausgelassen.
# ---------------------------------------------------------------------------
autoload -Uz add-zle-hook-widget add-zsh-hook
typeset -g _kompass_live_last=$'\x00'
_kompass_live() {
  [[ "$BUFFER" == "$_kompass_live_last" ]] && return
  _kompass_live_last="$BUFFER"
  [[ "$BUFFER$PWD" == *[$'\e\a\x1f']* ]] && return
  local REPLY
  _kompass_known "$BUFFER"
  print -rn -- $'\e]7771;'"${PWD}"$'\x1f'"${REPLY}"$'\x1f'"${BUFFER}"$'\a'
}
add-zle-hook-widget line-pre-redraw _kompass_live
add-zle-hook-widget line-init _kompass_live

# Befehl fertig: Rückgabewert und Ordner melden (OSC 7772) – für Fehlerhilfe und Missionen.
# Steht als Erstes in precmd_functions, damit $? noch unverfälscht ist.
_kompass_precmd() {
  local ec=$?
  print -rn -- $'\e]7772;'"${ec}"$'\x1f'"${PWD}"$'\a'
}
precmd_functions=(_kompass_precmd ${precmd_functions:#_kompass_precmd})
