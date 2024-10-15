// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            list_files,
            scan_files,
            listen_for_incomming_connect,
            list_devices,
            join_network,
            list_objects,
            get_thumbnail
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn join_network(ip: String) -> String {
    server::request_offer(ip.to_string()).await
}

mod database;
mod file;
mod server;
mod transport;

#[tauri::command]
async fn listen_for_incomming_connect() {
    println!("Starting server");
    server::start().await;
}

#[tauri::command()]
fn list_devices() -> String {
    serde_json::to_string(&server::list_all_connected_devices()).unwrap()
}
#[tauri::command]
fn list_objects(query: &str, path: &str) -> String {
    let database = database::Database::new(path);
    serde_json::to_string(&database.list_objects(query)).unwrap()
}

use serde::{Deserialize, Serialize};
use std::string::String;

#[derive(Serialize, Deserialize, Debug)]
struct Image {
    path: String,
    encoded: String,
}

#[tauri::command]
fn list_files(path: &str, query: &str, limit: usize, offset: usize, scan: bool) -> String {
    if scan {
        scan_files(path.to_string(), path);
    }
    let database = database::Database::new(path);
    let photos = database.list_photos(query, offset, limit);
    serde_json::to_string(&photos).unwrap()
}

#[tauri::command]
fn scan_files(directory: String, path: &str) {
    println!("Scanning folder {}", directory);
    file::scan_folder(directory, path);
}

#[tauri::command]
fn get_thumbnail(path: String) -> String {
    println!("Generating thumnail for {}", path);
    file::get_thumbnail(path)
}
