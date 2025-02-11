// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    #[cfg(debug_assertions)]
    let devtools = tauri_plugin_devtools::init(); // initialize the plugin as early as possible
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init());
    builder = builder.plugin(devtools); // then register it with Tauri

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

