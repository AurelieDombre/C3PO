#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use std::process::{Command, Stdio};
    use std::time::Duration;
    use std::thread;
    use std::sync::{Arc, Mutex};
    use std::fs::OpenOptions;
    use tauri::Manager;

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())

        // ← invoke_handler OBLIGATOIRE pour que useOllama fonctionne
        .invoke_handler(tauri::generate_handler![is_ollama_available])

        .setup(|app| {

            // =========================
            // LOG FILE
            // =========================
            let log_path = "C:/Users/Public/c3po_debug.log";
            let mut log = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
                .ok();

            macro_rules! log {
                ($($arg:tt)*) => {
                    if let Some(ref mut f) = log {
                        use std::io::Write;
                        let _ = writeln!(f, $($arg)*);
                    }
                };
            }

            log!("=== C3PO démarrage ===");

            // =========================
            // OLLAMA CHECK
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
            let ollama_ready = if ollama_available { true } else { wait_for_ollama(5) };
            log!("Ollama ready : {}", ollama_ready);

            // ← state partagé avec le frontend via invoke
            app.manage(OllamaState { available: ollama_ready });

            // =========================
            // BACKEND EXTERNE
            // =========================
            let backend_path = [
                "bin/start_backend/start_backend.exe",
                "start_backend.exe",
            ]
            .into_iter()
            .find_map(|candidate| {
                match app
                    .path()
                    .resolve(candidate, tauri::path::BaseDirectory::Resource)
                {
                    Ok(path) => {
                        log!(
                            "Backend candidate : {:?} (exists: {})",
                            path,
                            path.exists()
                        );

                        if path.exists() {
                            Some(path)
                        } else {
                            None
                        }
                    }
                    Err(e) => {
                        log!("ERREUR resolve backend {} : {:?}", candidate, e);
                        None
                    }
                }
            })
            .expect("backend introuvable");

            let backend_dir = backend_path.parent().unwrap().to_path_buf();
            log!("Backend path : {:?}", backend_path);
            log!("Fichier existe : {}", backend_path.exists());

            let backend_process: Arc<Mutex<Option<std::process::Child>>> =
                Arc::new(Mutex::new(None));
            let process_ref = backend_process.clone();

            // Lance le backend
            let child = Command::new(&backend_path)
                .current_dir(&backend_dir)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();

            match child {
                Ok(c) => {
                    log!("Backend spawné (PID {})", c.id());
                    *process_ref.lock().unwrap() = Some(c);
                    let ok = wait_for_backend(20);
                    log!("Port 8000 joignable : {}", ok);
                }
                Err(e) => {
                    log!(
                        "ERREUR spawn : {:?} | backend: {:?} | cwd: {:?}",
                        e,
                        backend_path,
                        backend_dir
                    );
                }
            }

            // =========================
            // MONITOR THREAD (redémarre si crash)
            // =========================
            let process_ref2 = backend_process.clone();
            let backend_path2 = backend_path.clone();
            let backend_dir2 = backend_dir.clone();

            thread::spawn(move || {
                let mut retries = 0;
                loop {
                    thread::sleep(Duration::from_millis(2000));

                    if std::net::TcpStream::connect("127.0.0.1:8000").is_ok() {
                        retries = 0;
                        continue;
                    }

                    if retries >= 3 { break; }
                    retries += 1;

                    if let Ok(mut guard) = process_ref2.lock() {
                        if let Some(child) = guard.as_mut() {
                            let _ = child.kill();
                        }
                        *guard = None;
                    }

                    if let Err(e) = Command::new(&backend_path2)
                        .current_dir(&backend_dir2)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                    {
                        log!(
                            "ERREUR respawn : {:?} | backend: {:?} | cwd: {:?}",
                            e,
                            backend_path2,
                            backend_dir2
                        );
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running app");
}

// =========================
// STATE OLLAMA
// =========================
#[derive(Clone)]
struct OllamaState {
    available: bool,
}

// =========================
// COMMANDE REACT
// =========================
#[tauri::command]
fn is_ollama_available(state: tauri::State<OllamaState>) -> bool {
    state.available
}
