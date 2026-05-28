#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::Duration;

    use tauri::Manager;

    // =========================
    // BUILD APP
    // =========================
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())

        .invoke_handler(tauri::generate_handler![
            is_ollama_available
        ])

        .setup(|app| {
            // =========================
            // LOG FILE
            // =========================
            let log_path = "C:/Users/Public/c3po_debug.log";
            let mut log_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
                .ok();

            macro_rules! log {
                ($($arg:tt)*) => {
                    if let Some(ref mut f) = log_file {
                        let _ = writeln!(f, $($arg)*);
                    }
                };
            }

            log!("==============================");
            log!("C3PO START");

            // =========================
            // OLLAMA CHECK
            // =========================
            fn is_ollama_running() -> bool {
                std::net::TcpStream::connect("127.0.0.1:11434").is_ok()
            }

            fn wait_for_backend() -> bool {
                for _ in 0..20 {
                    if std::net::TcpStream::connect("127.0.0.1:8000").is_ok() {
                        return true;
                    }
                    thread::sleep(Duration::from_millis(250));
                }
                false
            }

            let ollama_ready = is_ollama_running();

            app.manage(OllamaState {
                available: ollama_ready,
            });

            log!("Ollama ready: {}", ollama_ready);

            // petit délai stabilisation
            thread::sleep(Duration::from_millis(500));

            // =========================
            // BACKEND PATH SAFE RESOLVE
            // =========================
            let backend_path = match app.path().resolve(
                "bin/start_backend/start_backend.exe",
                tauri::path::BaseDirectory::Resource,
            ) {
                Ok(p) => p,
                Err(e) => {
                    log!("❌ resolve backend error: {:?}", e);
                    return Ok(());
                }
            };

            log!("Backend path: {:?}", backend_path);

            if !backend_path.exists() {
                log!("❌ backend exe NOT FOUND");
                return Ok(());
            }

            // =========================
            // WORKING DIRECTORY (IMPORTANT PYINSTALLER)
            // =========================
            let backend_dir = backend_path
                .parent()
                .unwrap()
                .to_path_buf();

            let internal_dir = backend_dir.join("_internal");

            let working_dir = if internal_dir.exists() {
                backend_dir.clone()
            } else {
                backend_dir.clone()
            };

            log!("Working dir: {:?}", working_dir);

            // =========================
            // SPAWN BACKEND
            // =========================
            let backend_result = Command::new(&backend_path)
                .current_dir(&working_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match backend_result {
                Ok(child) => {
                    log!("Backend started PID: {}", child.id());

                    // on ne bloque pas
                    std::mem::forget(child);

                    let ok = wait_for_backend();
                    log!("Backend reachable: {}", ok);
                }
                Err(e) => {
                    log!("❌ spawn error: {:?}", e);
                }
            }

            log!("Setup complete");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Clone)]
struct OllamaState {
    available: bool,
}

#[tauri::command]
fn is_ollama_available(state: tauri::State<OllamaState>) -> bool {
    state.available
}