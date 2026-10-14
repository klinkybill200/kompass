/* Kompass – Webseite: Sprachen (EN/DE) und die Live-Demo im Hero. */
(() => {
  "use strict";

  // ───────────────────────── Texte ─────────────────────────
  const T = {
    en: {
      "nav.why": "Why", "nav.features": "Features", "nav.showcase": "Showcase", "nav.download": "Download", "nav.support": "Support", "nav.cta": "Download",
      "hero.pill": "Free for macOS · Version 0.4",
      "hero.title": "The terminal that<br /><em>has your back.</em>",
      "hero.lead": "Kompass explains every command before it runs, stops the dangerous ones and understands plain English. Made for people who are afraid of the terminal – and for pros who want to move faster.",
      "hero.download": "Download for macOS", "hero.see": "See it in action",
      "hero.meta": "Apple Silicon · macOS 11 or later · 4.6 MB · signed & notarized by Apple",
      "demo.guard": "Guard active", "demo.tab": "›_ ~/Projects",
      "scene.plain": "Plain language", "scene.guard": "Guard & undo", "scene.explain": "Live explanations", "scene.help": "Error help",
      "why.eyebrow": "Why we build Kompass",
      "why.title": "The most powerful tool on your Mac shouldn't feel like a minefield.",
      "why.1.t": "The terminal is scary", "why.1.p": "A blinking cursor, cryptic commands, no undo. Most people never dare to try – and miss out on how fast and powerful it really is.",
      "why.2.t": "One typo, gone forever", "why.2.p": "rm deletes without a Trash. A single > can wipe a file. The classic terminal assumes you already know everything.",
      "why.3.t": "Kompass changes that", "why.3.p": "It explains, it protects, it teaches. A real zsh with a friendly co-pilot – and when you have learned enough, it simply gets out of your way.",
      "why.quote": "Our goal: anyone should be able to open a terminal without fear – and enjoy it.",
      "feat.eyebrow": "What Kompass does", "feat.title": "A real terminal, with a safety net.",
      "f.guard.t": "Command guard", "f.guard.p": "Every line is checked before it runs and rated safe, careful or dangerous. Risky commands get a friendly dialog that says in plain words what will happen.",
      "f.trash.t": "rm goes to the Trash", "f.trash.p": "Deleting is never final. Files land in the macOS Trash, and one click on “Undo” puts them back exactly where they were.",
      "f.plain.t": "Plain language", "f.plain.p": "Type “find all pdf files” – Kompass turns it into the right command, shows it to you and only runs it on your second Enter. English and German.",
      "f.explain.t": "Live explanations", "f.explain.p": "While you type, a bar below the terminal explains every part of the command – flags, files, pipes and redirects.",
      "f.help.t": "Error help", "f.help.p": "“No such file or directory”? Kompass translates cryptic errors, spots typos and offers the corrected command with one click.",
      "f.missions.t": "13 learning missions", "f.missions.p": "From “Where am I?” to scripts and pipes: short guided missions that check your real commands and celebrate your progress.",
      "f.tabs.t": "Tabs & shortcuts", "f.tabs.p": "⌘T for a new tab in the same folder, ⌘1–9 to jump, ⌘K to clear. Everything you expect from a modern Mac app.",
      "f.ai.t": "Optional AI", "f.ai.p": "For requests the built-in rules don't know, Kompass can ask Claude – only if you add your own API key. Without it, everything stays on your Mac.",
      "sc.eyebrow": "Showcase", "sc.title": "Learn by doing – one mission at a time.",
      "sc.p1": "Kompass comes with 13 missions in three levels: getting started, intermediate and pro. Each step checks the command you actually ran – no quizzes, no theory, just real practice in a safe playground.",
      "sc.l1": "Pick any mission, in any order", "sc.l2": "“Insert” puts the command into the line for you", "sc.l3": "Progress is saved, confetti included",
      "sc.toast": "Done: More details, please", "sc.side": "Missions", "sc.progress": "Your progress", "sc.g1": "Getting started", "sc.g3": "Pro", "sc.active": "active",
      "m.where": "Where am I?", "m.say": "Just say it", "m.step": "Another question", "m.hint": "how much disk space is free", "m.play": "Your own playground", "m.perm": "Understanding permissions",
      "aud.b.eyebrow": "For beginners", "aud.b.t": "Learn without fear",
      "aud.b.1": "Every command explained before it runs", "aud.b.2": "Dangerous actions need your OK", "aud.b.3": "Deleted files can always come back", "aud.b.4": "Missions that make learning fun",
      "aud.p.eyebrow": "For pros", "aud.p.t": "Move faster",
      "aud.p.1": "Your own zsh, your .zshrc, your aliases", "aud.p.2": "Plain language for the commands you never remember", "aud.p.3": "Tabs and native shortcuts", "aud.p.4": "A safety net for those late-night rm -rf moments",
      "privacy": "Runs <b>100 % locally</b> on your Mac. No account, no analytics, no tracking. The optional AI only receives your request when you add your own API key.",
      "dl.eyebrow": "Download", "dl.title": "Get Kompass – it's free.",
      "dl.req": "macOS 11 Big Sur or later · Apple Silicon (M1–M4) · 4.6 MB", "dl.notarized": "Signed with an Apple Developer ID and notarized by Apple", "dl.button": "Download .dmg",
      "dl.how": "How to install",
      "dl.s1": "Open the downloaded file and drag Kompass into the Applications folder.",
      "dl.s2": "Start Kompass – it is signed and notarized by Apple, so it opens without any security warning.",
      "dl.intel": "Intel Macs and Windows are not supported yet.",
      "sup.eyebrow": "Kompass is freeware", "sup.title": "Help keep Kompass alive.",
      "sup.p": "Kompass is and stays free – no ads, no subscription. If it makes the terminal easier for you, a small donation helps the developer keep the project actively maintained and growing.",
      "sup.5": "a coffee", "sup.10": "a pizza slice", "sup.20": "a whole evening of coding", "sup.via": "Secure one-time donation via PayPal.",
      "faq.title": "Questions",
      "faq.1.q": "Is Kompass really free?", "faq.1.a": "Yes. Kompass is freeware – no trial, no subscription, no ads. Donations are welcome but never required.",
      "faq.2.q": "Does it replace the macOS Terminal?", "faq.2.a": "It can. Kompass runs a real zsh with your own configuration, so all your commands, aliases and tools work as usual – with explanations and a safety net on top.",
      "faq.3.q": "Is my data sent anywhere?", "faq.3.a": "No. Everything runs locally. Only if you add your own Anthropic API key, requests that the built-in rules don't understand are sent to Claude – together with the folder path and file names of the current folder.",
      "faq.4.q": "Can I still really delete files?", "faq.4.a": "Yes. The guard dialog has a “Delete permanently” option. By default, though, rm moves files to the Trash so you can always undo.",
      "faq.5.q": "Is Kompass safe to install?", "faq.5.a": "Yes. Kompass is signed with an Apple Developer ID and notarized by Apple – macOS checks it before the first start, and it opens without warnings. The full source code is public on GitHub.",
      "footer.made": "Made with care for everyone who ever feared the terminal.",
      "meta.title": "Kompass – The terminal that has your back",
      "nav.github": "GitHub", "nav.contact": "Contact", "footer.legal": "Legal notice", "footer.contact": "Contact", "footer.source": "Source code",
      "hero.pill2": "Free & open source · macOS · v0.4",
      "dl.source": "View source on GitHub", "dl.releases": "All releases",
      "faq.6.q": "Is Kompass open source?", "faq.6.a": "Yes. The full source code is on GitHub under the MIT license. Ideas, bug reports and pull requests are very welcome.",
      "legal.title": "Legal notice & privacy – Kompass", "contact.title": "Contact – Kompass",
      "contact.h": "Get in touch", "contact.lead": "Questions, ideas, praise or a bug? I'd love to hear from you.",
      "contact.mail.t": "Email", "contact.mail.p": "The fastest way to reach me personally.",
      "contact.issue.t": "Bug or idea?", "contact.issue.p": "Open an issue on GitHub – that way others can follow along and chime in.", "contact.issue.b": "Open GitHub issues",
      "contact.code.t": "Contribute", "contact.code.p": "Kompass is open source (MIT). Fork it, improve it, send a pull request.", "contact.code.b": "Go to the repository",
      "contact.support.t": "Support the project", "contact.support.p": "Kompass is free. A small donation helps keep it maintained.", "contact.support.b": "Donate",
      "back": "← Back to the homepage",
    },
    de: {
      "nav.why": "Warum", "nav.features": "Funktionen", "nav.showcase": "Showcase", "nav.download": "Download", "nav.support": "Unterstützen", "nav.cta": "Download",
      "hero.pill": "Kostenlos für macOS · Version 0.4",
      "hero.title": "Das Terminal, das<br /><em>auf dich aufpasst.</em>",
      "hero.lead": "Kompass erklärt jeden Befehl, bevor er läuft, stoppt die gefährlichen und versteht ganz normale Sätze. Für alle, die Respekt vorm Terminal haben – und für Profis, die schneller sein wollen.",
      "hero.download": "Für macOS laden", "hero.see": "In Aktion sehen",
      "hero.meta": "Apple Silicon · ab macOS 11 · 4,6 MB · von Apple signiert & notarisiert",
      "demo.guard": "Wächter aktiv", "demo.tab": "›_ ~/Projekte",
      "scene.plain": "Klarsprache", "scene.guard": "Wächter & Rückgängig", "scene.explain": "Live-Erklärung", "scene.help": "Fehlerhilfe",
      "why.eyebrow": "Warum wir Kompass bauen",
      "why.title": "Das mächtigste Werkzeug auf deinem Mac sollte sich nicht wie ein Minenfeld anfühlen.",
      "why.1.t": "Das Terminal macht Angst", "why.1.p": "Ein blinkender Cursor, kryptische Befehle, kein Rückgängig. Die meisten trauen sich nie – und verpassen, wie schnell und mächtig es eigentlich ist.",
      "why.2.t": "Ein Tippfehler, alles weg", "why.2.p": "rm löscht ohne Papierkorb. Ein einziges > kann eine Datei leeren. Das klassische Terminal setzt voraus, dass du schon alles weißt.",
      "why.3.t": "Kompass ändert das", "why.3.p": "Es erklärt, es schützt, es bringt dir etwas bei. Eine echte zsh mit freundlichem Co-Piloten – und wenn du genug gelernt hast, hält es sich einfach zurück.",
      "why.quote": "Unser Ziel: Jeder soll ein Terminal ohne Angst öffnen können – und Spaß daran haben.",
      "feat.eyebrow": "Was Kompass kann", "feat.title": "Ein echtes Terminal, mit Sicherheitsnetz.",
      "f.guard.t": "Befehls-Wächter", "f.guard.p": "Jede Zeile wird vor dem Ausführen geprüft und als sicher, aufpassen oder gefährlich eingestuft. Riskante Befehle bekommen einen freundlichen Dialog, der in Klartext sagt, was passiert.",
      "f.trash.t": "rm landet im Papierkorb", "f.trash.p": "Löschen ist nie endgültig. Dateien wandern in den macOS-Papierkorb, und ein Klick auf „Rückgängig“ legt sie genau dorthin zurück, wo sie waren.",
      "f.plain.t": "Klarsprache", "f.plain.p": "Tippe „finde alle pdf dateien“ – Kompass macht den richtigen Befehl daraus, zeigt ihn dir und führt ihn erst beim zweiten Enter aus. Auf Deutsch und Englisch.",
      "f.explain.t": "Live-Erklärung", "f.explain.p": "Schon beim Tippen erklärt eine Leiste unter dem Terminal jeden Teil des Befehls – Optionen, Dateien, Pipes und Umleitungen.",
      "f.help.t": "Fehlerhilfe", "f.help.p": "„No such file or directory“? Kompass übersetzt kryptische Fehler, erkennt Tippfehler und bietet den korrigierten Befehl per Klick an.",
      "f.missions.t": "13 Lern-Missionen", "f.missions.p": "Von „Wo bin ich?“ bis zu Skripten und Pipes: kurze geführte Missionen, die deine echten Befehle prüfen und deine Fortschritte feiern.",
      "f.tabs.t": "Tabs & Kürzel", "f.tabs.p": "⌘T für einen neuen Tab im selben Ordner, ⌘1–9 zum Springen, ⌘K zum Aufräumen. Alles, was du von einer modernen Mac-App erwartest.",
      "f.ai.t": "KI – optional", "f.ai.p": "Für Wünsche, die das eingebaute Regelwerk nicht kennt, kann Kompass Claude fragen – nur mit deinem eigenen API-Schlüssel. Ohne bleibt alles auf deinem Mac.",
      "sc.eyebrow": "Showcase", "sc.title": "Lernen durch Tun – eine Mission nach der anderen.",
      "sc.p1": "Kompass bringt 13 Missionen in drei Stufen mit: Einstieg, Fortgeschritten und Profi. Jeder Schritt prüft den Befehl, den du wirklich ausgeführt hast – kein Quiz, keine Theorie, sondern echte Praxis auf einem sicheren Spielplatz.",
      "sc.l1": "Jede Mission frei wählbar, in beliebiger Reihenfolge", "sc.l2": "„Einsetzen“ schreibt den Befehl für dich in die Zeile", "sc.l3": "Fortschritt wird gespeichert – Konfetti inklusive",
      "sc.toast": "Geschafft: Mehr Details, bitte", "sc.side": "Missionen", "sc.progress": "Dein Fortschritt", "sc.g1": "Einstieg", "sc.g3": "Profi", "sc.active": "läuft",
      "m.where": "Wo bin ich?", "m.say": "Sag es einfach", "m.step": "Noch eine Frage", "m.hint": "wie viel speicherplatz ist frei", "m.play": "Dein eigener Spielplatz", "m.perm": "Rechte verstehen",
      "aud.b.eyebrow": "Für Einsteiger", "aud.b.t": "Lernen ohne Angst",
      "aud.b.1": "Jeder Befehl wird erklärt, bevor er läuft", "aud.b.2": "Gefährliches braucht dein OK", "aud.b.3": "Gelöschte Dateien kommen immer zurück", "aud.b.4": "Missionen, die Lernen zum Spaß machen",
      "aud.p.eyebrow": "Für Profis", "aud.p.t": "Schneller ans Ziel",
      "aud.p.1": "Deine eigene zsh, deine .zshrc, deine Aliase", "aud.p.2": "Klarsprache für die Befehle, die man sich nie merkt", "aud.p.3": "Tabs und native Tastenkürzel", "aud.p.4": "Ein Sicherheitsnetz für die rm -rf-Momente spät in der Nacht",
      "privacy": "Läuft <b>zu 100 % lokal</b> auf deinem Mac. Kein Konto, keine Analyse, kein Tracking. Die optionale KI bekommt deine Anfrage nur, wenn du einen eigenen API-Schlüssel hinterlegst.",
      "dl.eyebrow": "Download", "dl.title": "Hol dir Kompass – kostenlos.",
      "dl.req": "ab macOS 11 Big Sur · Apple Silicon (M1–M4) · 4,6 MB", "dl.notarized": "Mit Apple-Developer-ID signiert und von Apple notarisiert", "dl.button": ".dmg herunterladen",
      "dl.how": "So installierst du Kompass",
      "dl.s1": "Öffne die heruntergeladene Datei und zieh Kompass in den Programme-Ordner.",
      "dl.s2": "Kompass starten – die App ist von Apple signiert und notarisiert und öffnet sich ohne Sicherheitswarnung.",
      "dl.intel": "Intel-Macs und Windows werden noch nicht unterstützt.",
      "sup.eyebrow": "Kompass ist Freeware", "sup.title": "Hilf mit, Kompass am Leben zu halten.",
      "sup.p": "Kompass ist und bleibt kostenlos – ohne Werbung, ohne Abo. Wenn es dir den Weg ins Terminal leichter macht, hilft eine kleine Spende dem Entwickler, das Projekt aktiv zu pflegen und weiterzuentwickeln.",
      "sup.5": "ein Kaffee", "sup.10": "ein Stück Pizza", "sup.20": "ein ganzer Abend Programmieren", "sup.via": "Sichere einmalige Spende per PayPal.",
      "faq.title": "Fragen",
      "faq.1.q": "Ist Kompass wirklich kostenlos?", "faq.1.a": "Ja. Kompass ist Freeware – keine Testversion, kein Abo, keine Werbung. Spenden sind willkommen, aber nie Pflicht.",
      "faq.2.q": "Ersetzt es das macOS-Terminal?", "faq.2.a": "Kann es. Kompass startet eine echte zsh mit deiner eigenen Konfiguration – alle Befehle, Aliase und Werkzeuge funktionieren wie gewohnt, nur mit Erklärungen und Sicherheitsnetz.",
      "faq.3.q": "Werden meine Daten irgendwohin geschickt?", "faq.3.a": "Nein. Alles läuft lokal. Nur wenn du einen eigenen Anthropic-API-Schlüssel hinterlegst, gehen Wünsche, die das Regelwerk nicht versteht, an Claude – zusammen mit dem Ordnerpfad und den Dateinamen des aktuellen Ordners.",
      "faq.4.q": "Kann ich trotzdem endgültig löschen?", "faq.4.a": "Ja. Der Wächter-Dialog hat dafür „Endgültig löschen“. Standardmäßig legt rm aber alles in den Papierkorb, damit du es zurückholen kannst.",
      "faq.5.q": "Ist Kompass sicher zu installieren?", "faq.5.a": "Ja. Kompass ist mit einer Apple-Developer-ID signiert und von Apple notarisiert – macOS prüft die App vor dem ersten Start, und sie öffnet sich ohne Warnung. Der komplette Quellcode ist öffentlich auf GitHub.",
      "footer.made": "Mit Liebe gemacht – für alle, die je Respekt vorm Terminal hatten.",
      "meta.title": "Kompass – Das Terminal, das auf dich aufpasst",
      "nav.github": "GitHub", "nav.contact": "Kontakt", "footer.legal": "Impressum & Datenschutz", "footer.contact": "Kontakt", "footer.source": "Quellcode",
      "hero.pill2": "Kostenlos & Open Source · macOS · v0.4",
      "dl.source": "Quellcode auf GitHub", "dl.releases": "Alle Versionen",
      "faq.6.q": "Ist Kompass Open Source?", "faq.6.a": "Ja. Der komplette Quellcode liegt auf GitHub, unter der MIT-Lizenz. Ideen, Fehlermeldungen und Pull Requests sind herzlich willkommen.",
      "legal.title": "Impressum & Datenschutz – Kompass", "contact.title": "Kontakt – Kompass",
      "contact.h": "Schreib mir", "contact.lead": "Fragen, Ideen, Lob oder einen Fehler gefunden? Ich freue mich, von dir zu hören.",
      "contact.mail.t": "E-Mail", "contact.mail.p": "Der schnellste Weg, mich persönlich zu erreichen.",
      "contact.issue.t": "Fehler oder Idee?", "contact.issue.p": "Eröffne ein Issue auf GitHub – so können andere mitlesen und mitdiskutieren.", "contact.issue.b": "GitHub-Issues öffnen",
      "contact.code.t": "Mitmachen", "contact.code.p": "Kompass ist Open Source (MIT). Forken, verbessern, Pull Request schicken.", "contact.code.b": "Zum Repository",
      "contact.support.t": "Projekt unterstützen", "contact.support.p": "Kompass ist kostenlos. Eine kleine Spende hilft, es weiter zu pflegen.", "contact.support.b": "Spenden",
      "back": "← Zurück zur Startseite",
    },
  };

  // Demo-Inhalte je Sprache
  const D = {
    en: {
      tip: "Just start typing – Kompass explains every command before you press Enter.",
      tipLabel: "Tip", happens: "What happens", nl: "Plain language",
      safe: "safe", danger: "dangerous", enterTranslates: "Enter translates",
      plain: {
        input: "find all pdf files",
        sum: "Finds all PDF files here and in all subfolders",
        becomes: "becomes",
        toastEb: "Plain language · find all pdf files", toastText: "Enter runs the command.",
        out: ["./Invoices/2026-03.pdf", "./Invoices/2026-04.pdf", "./Print/Flyer_A5.pdf", "./Contract signed.pdf"],
      },
      guard: {
        sum: "Deletes the folder “Photos” – in Kompass it goes to the Trash",
        chips: [["rm", "Deletes files", "cmd"], ["-r", "also folders with content", "flag"], ["Photos", "folder", "arg"]],
        eb: "Kompass paused for a moment", title: "Move to the Trash?",
        fTitle: "Delete files",
        fText: "Normally this is permanent. Kompass moves it to the Trash instead – you can get it back any time.",
        meta: "Folder · 128 items", cancel: "Cancel", trash: "Move to Trash",
        out1: "🗑  Folder “Photos” is now in the Trash.", out2: "   Undo: type kompass-undo – or click Undo.",
        toast: "“Photos” is in the Trash", toastSub: "Nothing is lost.", undo: "Undo",
        back: "↩  “Photos” is back: ~/Projects/Photos", backToast: "“Photos” is back",
      },
      explain: {
        sum: "Shows what is in “Downloads” – as a detailed list, including hidden files, readable sizes",
        chips: [["ls", "Shows folder contents", "cmd"], ["-lah", "detailed list, hidden files, readable sizes", "flag"], ["Downloads", "folder", "arg"]],
        out: ["total 184M", "drwx------  14 you  staff   448B Oct 14 09:12 .", "-rw-r--r--   1 you  staff   4.6M Oct 13 18:40 Kompass-0.4.0-arm64.dmg", "-rw-r--r--   1 you  staff   2.1M Oct 12 11:03 Menu.pdf"],
      },
      help: {
        err: "cd: no such file or directory: photos",
        eb: "What happened? · cd photos", title: "Not found",
        text: "“photos” does not exist here. But there is “Photos” – did you mean that? (Upper and lower case matter!)",
        btn: "Insert “cd Photos”", sum: "Changes into the folder “Photos”",
      },
    },
    de: {
      tip: "Tippe einfach los – Kompass erklärt jeden Befehl, noch bevor du Enter drückst.",
      tipLabel: "Tipp", happens: "Das passiert", nl: "Klarsprache",
      safe: "sicher", danger: "gefährlich", enterTranslates: "Enter übersetzt",
      plain: {
        input: "finde alle pdf dateien",
        sum: "Findet alle PDF-Dateien hier und in allen Unterordnern",
        becomes: "wird zu",
        toastEb: "Klarsprache · finde alle pdf dateien", toastText: "Enter führt den Befehl aus.",
        out: ["./Rechnungen/2026-03.pdf", "./Rechnungen/2026-04.pdf", "./Druck/Flyer_A5.pdf", "./Vertrag unterschrieben.pdf"],
      },
      guard: {
        sum: "Löscht den Ordner „Fotos“ – in Kompass landet er im Papierkorb",
        chips: [["rm", "Löscht Dateien", "cmd"], ["-r", "auch Ordner samt Inhalt", "flag"], ["Fotos", "Ordner", "arg"]],
        eb: "Kompass hat kurz angehalten", title: "Ab in den Papierkorb?",
        fTitle: "Dateien löschen",
        fText: "Normalerweise ist das endgültig. Kompass legt es stattdessen in den Papierkorb – du kannst es jederzeit zurückholen.",
        meta: "Ordner · 128 Einträge", cancel: "Abbrechen", trash: "In den Papierkorb legen",
        out1: "🗑  Ordner „Fotos“ liegt jetzt im Papierkorb.", out2: "   Rückgängig: tippe kompass-undo – oder klick auf Rückgängig.",
        toast: "„Fotos“ liegt im Papierkorb", toastSub: "Nichts ist verloren.", undo: "Rückgängig",
        back: "↩  „Fotos“ ist wieder da: ~/Projekte/Fotos", backToast: "„Fotos“ ist wieder da",
      },
      explain: {
        sum: "Zeigt, was in „Downloads“ liegt – als Liste mit Details, auch versteckte Dateien, Größen lesbar",
        chips: [["ls", "Zeigt Ordnerinhalt", "cmd"], ["-lah", "Liste, versteckte Dateien, lesbare Größen", "flag"], ["Downloads", "Ordner", "arg"]],
        out: ["total 184M", "drwx------  14 du  staff   448B 14 Okt 09:12 .", "-rw-r--r--   1 du  staff   4.6M 13 Okt 18:40 Kompass-0.4.0-arm64.dmg", "-rw-r--r--   1 du  staff   2.1M 12 Okt 11:03 Speisekarte.pdf"],
      },
      help: {
        err: "cd: no such file or directory: fotos",
        eb: "Was ist passiert? · cd fotos", title: "Nicht gefunden",
        text: "„fotos“ gibt es hier nicht. Es gibt aber „Fotos“ – meintest du das? (Groß- und Kleinschreibung zählt!)",
        btn: "„cd Fotos“ einsetzen", sum: "Wechselt in den Ordner „Fotos“",
      },
    },
  };

  // ───────────────────────── Sprache ─────────────────────────
  const params = new URLSearchParams(location.search);
  let lang = params.get("lang") === "de" ? "de" : params.get("lang") === "en" ? "en" : null;
  if (!lang) {
    try { lang = localStorage.getItem("kompass.site.lang"); } catch { /* egal */ }
  }
  if (lang !== "de") lang = "en"; // Hauptsprache: Englisch

  function applyLang(l) {
    lang = l;
    const dict = T[l];
    document.documentElement.lang = l;
    const pageTitle = document.body.dataset.title; // Unterseiten: eigener Titel je Sprache
    document.title = pageTitle ? dict[pageTitle] : dict["meta.title"];
    // Lange Texte (Impressum, Kontakt) stehen als ganze Blöcke je Sprache im HTML.
    document.querySelectorAll("[data-lang-block]").forEach((el) => { el.hidden = el.dataset.langBlock !== l; });
    document.querySelectorAll("[data-i18n]").forEach((el) => {
      const v = dict[el.dataset.i18n];
      if (v != null) el.textContent = v;
    });
    document.querySelectorAll("[data-i18n-html]").forEach((el) => {
      const v = dict[el.dataset.i18nHtml];
      if (v != null) el.innerHTML = v;
    });
    document.querySelectorAll(".lang button").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.lang === l)));
    try { localStorage.setItem("kompass.site.lang", l); } catch { /* egal */ }
    const url = new URL(location.href);
    if (l === "de") url.searchParams.set("lang", "de"); else url.searchParams.delete("lang");
    history.replaceState(null, "", url);
    if (term) startScene(current); // Demo in neuer Sprache neu starten (nur auf der Startseite)
  }

  document.querySelectorAll(".lang button").forEach((b) => b.addEventListener("click", () => applyLang(b.dataset.lang)));
  const year = document.getElementById("year");
  if (year) year.textContent = String(new Date().getFullYear());

  // ───────────────────────── Live-Demo ─────────────────────────
  const $ = (id) => document.getElementById(id);
  const term = $("demo-term"), explain = $("demo-explain"), toast = $("demo-toast"), dialog = $("demo-dialog"), guardPill = $("demo-guard");
  const sceneButtons = [...document.querySelectorAll(".scenes button")];
  const ORDER = ["plain", "guard", "explain", "help"];
  const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;
  let current = "plain";
  let token = 0; // jede neue Szene bekommt eine neue Nummer → alte Abläufe brechen ab
  let cwd = "~/Projects";

  const esc = (s) => s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
  class Cancelled extends Error {}
  const sleep = (ms, my) => new Promise((res, rej) => setTimeout(() => (my === token ? res() : rej(new Cancelled())), reduced ? Math.min(ms, 250) : ms));

  function promptHtml() {
    return `<span class="p-user">you@mac</span> <span class="p-dir">${esc(cwd)}</span> % `;
  }
  function newPrompt() {
    const line = document.createElement("div");
    line.innerHTML = promptHtml() + '<span class="input"></span><span class="caret"></span>';
    term.appendChild(line);
    trimTerm();
    return line.querySelector(".input");
  }
  function outLine(text, cls = "out") {
    const d = document.createElement("div");
    d.className = cls;
    d.textContent = text;
    term.appendChild(d);
    trimTerm();
  }
  function removeCaret() { term.querySelectorAll(".caret").forEach((c) => c.remove()); }
  function trimTerm() { while (term.children.length > 12) term.firstElementChild.remove(); }
  async function type(el, text, my, speed = 55) {
    for (const ch of text) {
      el.textContent += ch;
      await sleep(reduced ? 0 : speed + Math.random() * 40, my);
    }
  }
  function setTip() {
    const d = D[lang];
    explain.className = "explain";
    explain.innerHTML = `<div class="ex-head"><span class="badge">${d.tipLabel}</span><p class="ex-tip">${esc(d.tip)}</p></div>`;
  }
  function setExplain({ badge, cls = "", sum, level, levelCls = "", chips = [] }) {
    explain.className = `explain ${cls}`;
    explain.innerHTML =
      `<div class="ex-head"><span class="badge ${cls}">${esc(badge)}</span><p class="ex-sum">${esc(sum)}</p>` +
      `<span class="ex-level ${levelCls}">${level ? `<span class="dot"></span>${esc(level)}` : ""}</span></div>` +
      (chips.length ? `<div class="chips">${chips.map(([c, m, k]) => `<div class="chip ${k}"><code>${esc(c)}</code><span>${esc(m)}</span></div>`).join("")}</div>` : "");
  }
  function showToast(kind, icon, html) {
    toast.className = `demo-toast ${kind}`;
    toast.innerHTML = `<span class="ti">${icon}</span><div>${html}</div>`;
    toast.hidden = false;
  }
  function reset() {
    term.innerHTML = "";
    toast.hidden = true;
    dialog.hidden = true;
    guardPill.classList.remove("danger");
    cwd = lang === "de" ? "~/Projekte" : "~/Projects";
    setTip();
  }
  function setProgress(p) {
    const b = sceneButtons.find((x) => x.dataset.scene === current);
    if (b) b.style.setProperty("--progress", `${Math.round(p * 100)}%`);
  }

  const SCENES = {
    async plain(my) {
      const d = D[lang], s = d.plain;
      const input = newPrompt();
      await sleep(600, my);
      await type(input, s.input.slice(0, 9), my);
      setExplain({ badge: d.nl, cls: "nl", sum: s.sum, level: d.enterTranslates, chips: [] });
      await type(input, s.input.slice(9), my);
      setExplain({ badge: d.nl, cls: "nl", sum: s.sum, level: d.enterTranslates, chips: [[s.becomes, "", "cmd"]] });
      explain.querySelector(".chip").innerHTML = `<span>${esc(s.becomes)}</span><code>find . -type f -iname '*.pdf'</code>`;
      setProgress(0.3);
      await sleep(1400, my);
      // Enter: Satz wird zum Befehl
      input.innerHTML = `<span class="cmd-new">find . -type f -iname '*.pdf'</span>`;
      showToast("nl", "→", `<p class="muted">${esc(s.toastEb)}</p><p><span class="tc">find . -type f -iname '*.pdf'</span></p><p>${esc(s.sum)}. ${esc(s.toastText)}</p>`);
      setProgress(0.55);
      await sleep(2600, my);
      removeCaret();
      toast.hidden = true;
      for (const l of s.out) { outLine(l); await sleep(140, my); }
      setProgress(0.85);
      newPrompt();
      setTip();
      await sleep(2600, my);
    },

    async guard(my) {
      const d = D[lang], g = d.guard;
      const folder = lang === "de" ? "Fotos" : "Photos";
      const input = newPrompt();
      await sleep(600, my);
      await type(input, `rm -r ${folder}`, my);
      setExplain({ badge: d.happens, cls: "danger", sum: g.sum, level: d.danger, levelCls: "danger", chips: g.chips });
      setProgress(0.2);
      await sleep(1500, my);
      guardPill.classList.add("danger");
      dialog.innerHTML =
        `<div class="dlg"><div class="dlg-head"><span class="dlg-icon">🗑</span><div><p class="eb">${esc(g.eb)}</p><h4>${esc(g.title)}</h4></div></div>` +
        `<code class="cmd">rm -r ${folder}</code>` +
        `<p class="finding"><b>${esc(g.fTitle)}</b>${esc(g.fText)}</p>` +
        `<div class="target"><span>📁</span><span>${folder}</span><span>${esc(g.meta)}</span></div>` +
        `<div class="dlg-actions"><span>${esc(g.cancel)}</span><span class="primary">${esc(g.trash)}</span></div></div>`;
      dialog.hidden = false;
      setProgress(0.4);
      await sleep(2600, my);
      dialog.querySelector(".primary").classList.add("press");
      await sleep(260, my);
      dialog.hidden = true;
      guardPill.classList.remove("danger");
      removeCaret();
      input.innerHTML = `<span class="cmd-new">kompass-trash -r ${folder}</span>`;
      outLine(g.out1, "ok");
      outLine(g.out2, "out");
      setTip();
      showToast("ok", "🗑", `<p class="tt">${esc(g.toast)}</p><p class="muted">${esc(g.toastSub)}</p><span class="tb">${esc(g.undo)}</span>`);
      newPrompt();
      setProgress(0.7);
      await sleep(2200, my);
      toast.querySelector(".tb").classList.add("press");
      await sleep(260, my);
      showToast("ok", "✓", `<p class="tt">${esc(g.backToast)}</p>`);
      removeCaret();
      outLine(g.back, "ok");
      newPrompt();
      setProgress(0.9);
      await sleep(2400, my);
    },

    async explain(my) {
      const d = D[lang], x = d.explain;
      const input = newPrompt();
      await sleep(600, my);
      await type(input, "ls", my);
      setExplain({ badge: d.happens, sum: x.sum.split(" – ")[0], level: d.safe, chips: x.chips.slice(0, 1) });
      await type(input, " -lah", my);
      setExplain({ badge: d.happens, sum: x.sum, level: d.safe, chips: x.chips.slice(0, 2) });
      await type(input, " Downloads", my);
      setExplain({ badge: d.happens, sum: x.sum, level: d.safe, chips: x.chips });
      setProgress(0.45);
      await sleep(2400, my);
      removeCaret();
      for (const l of x.out) { outLine(l); await sleep(120, my); }
      newPrompt();
      setTip();
      setProgress(0.85);
      await sleep(2600, my);
    },

    async help(my) {
      const d = D[lang], h = d.help;
      const wrong = lang === "de" ? "fotos" : "photos";
      const right = lang === "de" ? "Fotos" : "Photos";
      let input = newPrompt();
      await sleep(600, my);
      await type(input, `cd ${wrong}`, my);
      await sleep(700, my);
      removeCaret();
      outLine(h.err, "err");
      input = newPrompt();
      showToast("help", "?", `<p class="muted">${esc(h.eb)}</p><p class="tt">${esc(h.title)}</p><p>${esc(h.text)}</p><span class="tb">${esc(h.btn)}</span>`);
      setProgress(0.35);
      await sleep(2800, my);
      toast.querySelector(".tb").classList.add("press");
      await sleep(260, my);
      toast.hidden = true;
      input.textContent = `cd ${right}`;
      setExplain({ badge: d.happens, sum: h.sum, level: d.safe, chips: [["cd", lang === "de" ? "Wechselt den Ordner" : "Changes the folder", "cmd"], [right, lang === "de" ? "Ordner" : "folder", "arg"]] });
      setProgress(0.6);
      await sleep(1600, my);
      removeCaret();
      cwd = `${cwd}/${right}`;
      newPrompt();
      setTip();
      setProgress(0.9);
      await sleep(2600, my);
    },
  };

  async function startScene(name) {
    current = name;
    const my = ++token;
    sceneButtons.forEach((b) => {
      const on = b.dataset.scene === name;
      b.classList.toggle("active", on);
      b.setAttribute("aria-selected", String(on));
      b.style.setProperty("--progress", "0%");
    });
    reset();
    try {
      await SCENES[name](my);
      setProgress(1);
      await sleep(400, my);
      // automatisch zur nächsten Szene
      startScene(ORDER[(ORDER.indexOf(name) + 1) % ORDER.length]);
    } catch (e) {
      if (!(e instanceof Cancelled)) throw e;
    }
  }

  sceneButtons.forEach((b) => b.addEventListener("click", () => startScene(b.dataset.scene)));
  // Interne Links behalten die Sprache bei (z. B. Startseite → Impressum).
  document.addEventListener("click", (e) => {
    const a = e.target.closest("a[href^='/']");
    if (!a || lang !== "de" || a.hasAttribute("download") || a.getAttribute("href").startsWith("/download")) return;
    const url = new URL(a.href, location.href);
    url.searchParams.set("lang", "de");
    a.href = url.pathname + url.search + url.hash;
  });

  applyLang(lang); // startet auch die Demo
})();
