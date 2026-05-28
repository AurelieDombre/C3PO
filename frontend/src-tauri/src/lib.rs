#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use std::process::{Command, Stdio};
    use std::time::Duration;
    use std::thread;
    use tauri::Manager;

    tauri::Builder::default()

        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())

        .invoke_handler(tauri::generate_handler![
            is_ollama_available
        ])

        .setup(|app| {

            // =========================
            // LOGS DEBUG
            // =========================
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // =========================
            // OLLAMA CHECK (FIABLE)
            // =========================
            fn is_ollama_running() -> bool {
                std::net::TcpStream::connect("127.0.0.1:11434").is_ok()
            }

            fn wait_for_ollama(max_tries: u32) -> bool {
                for _ in 0..max_tries {
                    if std::net::TcpStream::connect("127.0.0.1:11434").is_ok() {
                        return true;
                    }
                    thread::sleep(Duration::from_millis(500));
                }
                false
            }

            fn wait_for_backend(max_tries: u32) -> bool {
                for _ in 0..max_tries {
                    if std::net::TcpStream::connect("127.0.0.1:8000").is_ok() {
                        return true;
                    }
                    thread::sleep(Duration::from_millis(250));
                }
                false
            }

            let ollama_available = is_ollama_running();

            let ollama_ready = if ollama_available {
                true
            } else {
                wait_for_ollama(5)
            };

            // =========================
            // STATE GLOBAL
            // =========================
            app.manage(OllamaState {
                available: ollama_ready,
            });
            // wait minimal stabilisation runtime
            std::thread::sleep(std::time::Duration::from_millis(500));

            // =========================
            // BACKEND PYTHON
            // =========================
            let backend_path = app
                .path()
                .resolve(
                    "bin/start_backend/start_backend.exe",
                    tauri::path::BaseDirectory::Resource,
                )
                .expect("backend introuvable");

            println!("Backend path: {:?}", backend_path);
            println!("Launching backend...");

            let backend = Command::new(backend_path)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn();

            match backend {
                Ok(child) => {
                    std::mem::forget(child);
                    if wait_for_backend(20) {
                        println!("Backend started successfully");
                    } else {
                        eprintln!("Backend started but port 8000 is not reachable");
                    }
                }
                Err(e) => {
                    eprintln!("❌ Backend failed to start: {:?}", e);
                }
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
// COMMANDS REACT
// =========================
#[tauri::command]
fn is_ollama_available(state: tauri::State<OllamaState>) -> bool {
    state.available
}
