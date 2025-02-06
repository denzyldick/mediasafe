// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    #[cfg(debug_assertions)]
    //  let devtools = tauri_plugin_devtools::init(); // initialize the plugin as early as possible
    let builder = tauri::Builder::default()
        .setup(|app| {
            app.manage(App {
                database_path: "/home/denzyl/",
            })
        })
        .plugin(tauri_plugin_dialog::init());

    //    builder = builder.plugin(devtools); // then register it with Tauri

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct App {
    database_path: &'static str,
}

impl App{
    pub fn set_database_path(&mut self, path: &str){
        self.database_path = path;
    }
}
