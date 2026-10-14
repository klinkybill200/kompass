// `mod xyz;` bindet die Datei `xyz.rs` als Modul ein (ähnlich wie `import` in JS).
#[macro_use]
mod i18n;
mod errors;
mod explain;
mod guard;
mod ki;
mod nl;
mod pty;
pub mod trash_cli;
mod trashlog;

use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
// `Emitter` stellt `app.emit(...)` bereit (Nachricht ans Frontend schicken).
use tauri::Emitter;

/// Für den Kommandozeilen-Modus: zuletzt gewählte Sprache laden.
pub fn load_language() {
    i18n::load_saved();
}

/// Startet die Tauri-App.
/// `pub` heißt: von außen (main.rs) aufrufbar.
pub fn run() {
    tauri::Builder::default()
        // Gemeinsamer Zustand, auf den alle Befehle zugreifen (die laufenden Shells).
        .manage(pty::PtyState::default())
        .setup(|app| {
            build_menu(app)?;
            Ok(())
        })
        // Menüpunkt gewählt (auch per Tastenkürzel) → Frontend erledigt den Rest.
        .on_menu_event(|app, event| {
            let _ = app.emit("menu", event.id().0.clone());
        })
        // Diese Rust-Funktionen kann das Frontend per `invoke("name")` aufrufen.
        .invoke_handler(tauri::generate_handler![
            pty::pty_spawn,
            pty::pty_write,
            pty::pty_resize,
            pty::guard_answer,
            pty::pty_close,
            i18n::set_language,
            i18n::system_language,
            open_donation,
            ki::ki_status,
            ki::ki_set_key,
            ki::ki_clear_key,
            ki::ki_test,
            trashlog::trash_list,
            trashlog::trash_restore,
        ])
        .run(tauri::generate_context!())
        // `expect` bricht mit dieser Meldung ab, falls der Start scheitert.
        .expect("Kompass konnte nicht gestartet werden");
}

/// Öffnet die PayPal-Spendenseite im Browser.
/// Der Link wird hier fest zusammengebaut – das Frontend darf nur den Betrag wählen.
#[tauri::command]
fn open_donation(amount: u32) -> Result<(), String> {
    if ![5, 10, 20].contains(&amount) {
        return Err("Ungültiger Betrag".into());
    }
    let url = format!(
        "https://www.paypal.com/donate/?business=klinkybill%40gmail.com&amount={amount}&currency_code=USD&no_recurring=1&item_name=Kompass%20%28Freeware%29"
    );
    // `open` ist das macOS-Werkzeug, das Links im Standardbrowser öffnet.
    std::process::Command::new("open").arg(url).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

/// Das Mac-Menü oben am Bildschirmrand – mit den Tastenkürzeln für Tabs.
/// `&mut tauri::App` = veränderbarer Zugriff auf die App; `?` gibt Fehler weiter.
fn build_menu(app: &mut tauri::App) -> tauri::Result<()> {
    let h = app.handle();
    // Kleiner Helfer: Menüpunkt mit Kennung, Text und Tastenkürzel.
    let item = |id: &str, text: &str, key: &str| MenuItemBuilder::with_id(id, text).accelerator(key).build(h);

    let app_menu = SubmenuBuilder::new(h, "Kompass")
        .about(None)
        .separator()
        .item(&item("settings", "Einstellungen …", "CmdOrCtrl+,")?)
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()?;

    let shell = SubmenuBuilder::new(h, "Shell")
        .item(&item("tab-new", "Neuer Tab", "CmdOrCtrl+T")?)
        .item(&item("tab-close", "Tab schließen", "CmdOrCtrl+W")?)
        .separator()
        .item(&item("clear", "Fenster aufräumen", "CmdOrCtrl+K")?)
        .build()?;

    // Kopieren/Einfügen braucht diese Standard-Einträge, sonst gehen ⌘C/⌘V nicht.
    let edit = SubmenuBuilder::new(h, "Bearbeiten").undo().redo().separator().cut().copy().paste().select_all().build()?;

    let view = SubmenuBuilder::new(h, "Darstellung")
        .item(&item("panel-missions", "Missionen", "CmdOrCtrl+Shift+M")?)
        .item(&item("panel-trash", "Papierkorb", "CmdOrCtrl+Shift+P")?)
        .separator()
        .item(&item("font-bigger", "Schrift größer", "CmdOrCtrl+=")?)
        .item(&item("font-smaller", "Schrift kleiner", "CmdOrCtrl+-")?)
        .item(&item("font-reset", "Normale Schriftgröße", "CmdOrCtrl+0")?)
        .build()?;

    let mut window = SubmenuBuilder::new(h, "Fenster")
        .minimize()
        .separator()
        .item(&item("tab-prev", "Vorheriger Tab", "CmdOrCtrl+Shift+[")?)
        .item(&item("tab-next", "Nächster Tab", "CmdOrCtrl+Shift+]")?)
        .separator();
    // ⌘1 bis ⌘9 springen direkt zu einem Tab. `1..=9` = Zahlen 1 bis einschließlich 9.
    for n in 1..=9 {
        window = window.item(&item(&format!("tab-{n}"), &format!("Tab {n}"), &format!("CmdOrCtrl+{n}"))?);
    }
    let window = window.build()?;

    let menu = MenuBuilder::new(h).items(&[&app_menu, &shell, &edit, &view, &window]).build()?;
    app.set_menu(menu)?;
    Ok(())
}
