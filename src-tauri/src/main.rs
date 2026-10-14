// Verhindert unter Windows ein zusätzliches Konsolenfenster (auf dem Mac ohne Wirkung).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// `main` ist der Einstiegspunkt jedes Rust-Programms.
// Die eigentliche Logik steckt in der Bibliothek `kompass_lib` (siehe lib.rs).
fn main() {
    // Kommandozeilen-Argumente einsammeln. `skip(1)` überspringt den Programmnamen.
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Aufruf aus der Shell (`kompass-trash`, `kompass-undo`): nur Papierkorb, kein Fenster.
    if args.first().is_some_and(|a| a.starts_with("--kompass-")) {
        kompass_lib::load_language(); // gewählte Sprache für die Meldungen
    }
    match args.first().map(String::as_str) {
        Some("--kompass-trash") => std::process::exit(kompass_lib::trash_cli::trash(&args[1..])),
        Some("--kompass-undo") => std::process::exit(kompass_lib::trash_cli::undo()),
        _ => {}
    }

    kompass_lib::run()
}
