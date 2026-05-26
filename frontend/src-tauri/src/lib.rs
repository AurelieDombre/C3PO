#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

  tauri::Builder::default()

    // Plugin shell
    .plugin(tauri_plugin_shell::init()) //autorise React à ouvrir des fichiers/programmes
    .plugin(tauri_plugin_opener::init())

    // Plugin logs
    .setup(|app| {

      if cfg!(debug_assertions) {

        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      Ok(())
    })

    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}