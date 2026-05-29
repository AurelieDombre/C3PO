#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use std::process::{Command, Stdio};
    use std::time::Duration;
    use std::thread;
    use std::sync::{Arc, Mutex};
    use tauri::Manager;

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {

            let backend_path = app.path().resolve(
                "bin/start_backend/start_backend.exe",
                tauri::path::BaseDirectory::Resource,
            ).expect("backend introuvable");

            let backend_dir = backend_path.parent().unwrap().to_path_buf();

            // =========================
            // CLONES POUR THREADS
            // =========================
            let backend_path_1 = backend_path.clone();
            let backend_dir_1 = backend_dir.clone();

            let backend_path_2 = backend_path.clone();
            let backend_dir_2 = backend_dir.clone();

            // état partagé du process backend
            let backend_process = Arc::new(Mutex::new(None::<std::process::Child>));
            let process_ref = backend_process.clone();

            // =========================
            // START BACKEND FUNCTION
            // =========================
            let start_backend = move || {
                let mut guard = process_ref.lock().unwrap();

                let child = Command::new(&backend_path_1)
                    .current_dir(&backend_dir_1)
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn();

                match child {
                    Ok(c) => {
                        println!("[backend] started PID {}", c.id());
                        *guard = Some(c);
                        true
                    }
                    Err(e) => {
                        println!("[backend] spawn error: {:?}", e);
                        false
                    }
                }
            };

            // =========================
            // HEALTH CHECK
            // =========================
            let health_check = || -> bool {
                std::net::TcpStream::connect("127.0.0.1:8000").is_ok()
            };

            // =========================
            // START INITIAL BACKEND
            // =========================
            start_backend();

            // =========================
            // MONITOR THREAD
            // =========================
            let process_ref2 = backend_process.clone();

            thread::spawn(move || {
                let mut retries = 0;

                loop {
                    thread::sleep(Duration::from_millis(1500));

                    if health_check() {
                        retries = 0;
                        continue;
                    }

                    println!("[backend] DOWN detected");

                    if retries >= 3 {
                        println!("[backend] MAX retries reached");
                        break;
                    }

                    retries += 1;

                    // kill old process
                    if let Ok(mut guard) = process_ref2.lock() {
                        if let Some(child) = guard.as_mut() {
                            let _ = child.kill();
                        }
                        *guard = None;
                    }

                    println!("[backend] restarting... attempt {}", retries);

                    let _ = Command::new(&backend_path_2)
                        .current_dir(&backend_dir_2)
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .spawn();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running app");
}