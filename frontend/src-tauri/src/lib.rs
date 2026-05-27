#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use std::process::{Command, Stdio};
    use tauri::Manager;

    tauri::Builder::default()

        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())

        .setup(|app| {

            // logs debug
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // =========================
            // 1. DETECTION OLLAMA
            // =========================
            let ollama_available = Command::new("ollama")
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output()
                .is_ok();

            // on stocke le résultat dans l'état global Tauri
            app.manage(OllamaState {
                available: ollama_available,
            });
            let ollama_version = Command::new("ollama")
              .arg("--version")
              .output()
              .ok()
              .and_then(|o| String::from_utf8(o.stdout).ok());
            // =========================
            // 2. LANCEMENT BACKEND PYTHON
            // =========================
            let _backend = Command::new("backend.exe")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();

            if _backend.is_err() {
                eprintln!("Backend introuvable (backend.exe)");
            }

            Ok(())
        })

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


// =========================
// STATE GLOBAL
// =========================
#[derive(Clone)]
struct OllamaState {
    available: bool,
}


// =========================
// COMMANDS ACCESSIBLES DEPUIS REACT
// =========================
#[tauri::command]
fn is_ollama_available(state: tauri::State<OllamaState>) -> bool {
    state.available
}