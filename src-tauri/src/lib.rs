// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn get_last_scan_time(app: tauri::AppHandle) -> String {
    let path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let database = database::Database::new(&path);
    database
        .get_last_scan_time()
        .unwrap_or_else(|| "Never".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_devtools::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_initial_state,
            list_files,
            scan_files,
            get_last_scan_time,
            listen_for_incomming_connect,
            list_devices,
            remove_directory,
            add_directory,
            list_directories,
            join_network,
            list_objects,
            get_thumbnail,
            get_ip,
            get_device_by_name,
            get_heatmap_data,
            generate_dummy_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn join_network(ip: String) -> String {
    server::request_offer(ip.to_string()).await
}

mod config;
mod database;
mod device;
mod directory;
mod file;
mod server;
mod transport;

#[tauri::command]
async fn listen_for_incomming_connect(app: tauri::AppHandle) {
    println!("Starting server");
    let path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    server::start(path).await;
}
use get_if_addrs::get_if_addrs;
use serde_json::from_str;
use std::net::Ipv4Addr;
#[tauri::command()]
async fn get_ip() -> String {
    let ifaces = get_if_addrs().unwrap();

    let mut ip = String::from("");
    for iface in ifaces {
        if let std::net::IpAddr::V4(ipv4) = iface.ip() {
            // Check if the IP address is within the local network ranges
            if is_local_network_ip(ipv4) {
                ip = ipv4.to_string();
                println!("Local Network IP: {}", ipv4);
                break; // Assuming you only need one IP, otherwise remove this line
            }
        }
    }
    ip
}
// Helper function to check if an IPv4 address is in the private range
fn is_local_network_ip(ip: Ipv4Addr) -> bool {
    ip.octets()[0] == 10
        || (ip.octets()[0] == 172 && (16..=31).contains(&ip.octets()[1]))
        || (ip.octets()[0] == 192 && ip.octets()[1] == 168)
}

#[tauri::command()]
async fn list_devices(app: tauri::AppHandle) -> String {
    let path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let devices = device::list_devices(&path);
    serde_json::to_string(&devices).unwrap()
}
#[tauri::command]
async fn list_objects(app: tauri::AppHandle, query: String) -> String {
    let path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let database = database::Database::new(&path);
    serde_json::to_string(&database.list_objects(&query)).unwrap()
}

use serde::{Deserialize, Serialize};
use std::string::String;

#[derive(Serialize, Deserialize, Debug)]
struct Image {
    path: String,
    encoded: String,
}

use tauri::Manager;

#[tauri::command]
async fn list_files(
    app: tauri::AppHandle,
    query: String,
    limit: usize,
    offset: usize,
    scan: bool,
    favorites_only: bool,
) -> String {
    if scan {
        println!("Scanning for photos.");
        // Note: scan_files now requires app handle, so we skip it here
        // The frontend will call scan_files directly
    }
    let path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let database = database::Database::new(&path);
    let photos = database.list_photos(&query, offset, limit, favorites_only);
    println!("{} photos retrieved", photos.len());
    serde_json::to_string(&photos).unwrap()
}

#[tauri::command]
async fn scan_files(app: tauri::AppHandle) {
    let path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    use std::time::SystemTime;
    use tauri::Emitter;

    let directories = directory::list_directories(&path);
    let total_dirs = directories.len();

    for (index, directory) in directories.iter().enumerate() {
        println!(
            "Scanning folder {} ({}/{})",
            directory,
            index + 1,
            total_dirs
        );

        // Emit progress event
        let _ = app.emit(
            "scan-progress",
            serde_json::json!({
                "status": "scanning",
                "current_directory": directory,
                "progress": ((index + 1) as f32 / total_dirs as f32 * 100.0) as u32,
                "current": index + 1,
                "total": total_dirs
            }),
        );

        file::scan_folder(directory.clone(), &path);
    }

    // Save last scan timestamp
    let database_path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let database = database::Database::new(&database_path);
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
    database.set_last_scan_time(timestamp);

    // Emit completion event
    let _ = app.emit(
        "scan-progress",
        serde_json::json!({
            "status": "complete",
            "progress": 100
        }),
    );
}

#[tauri::command]
async fn get_thumbnail(path: String) -> String {
    println!("Generating thumnail for {}", path);
    file::get_thumbnail(path)
}

#[tauri::command]
async fn get_device_by_name(app: tauri::AppHandle, name: String) -> String {
    let path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let device = device::get_device_by_name(&name, &path);
    serde_json::to_string(&device).unwrap()
}

#[tauri::command]
async fn list_directories(app: tauri::AppHandle) -> String {
    let path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let directories = directory::list_directories(&path);
    serde_json::to_string(&directories).unwrap()
}
#[tauri::command]
async fn remove_directory(app: tauri::AppHandle, path: String) {
    let config_path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    directory::remove_directory(path, &config_path);
}
#[tauri::command]
async fn add_directory(app: tauri::AppHandle, path: String) {
    let config_path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    println!(
        "Command add_directory called with path: {} and config_path: {}",
        path, config_path
    );
    directory::add_directory(path, &config_path);
}
#[tauri::command]
async fn get_initial_state(app: tauri::AppHandle) -> String {
    let path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let database = database::Database::new(&path);
    let state = database.get_state();
    serde_json::to_string(&state).unwrap()
}

#[tauri::command]
async fn set_initial_state(app: tauri::AppHandle, state: String) {
    let state = from_str(&state).unwrap();
    let path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let database = database::Database::new(&path);
    database.set_state(state);
}

#[tauri::command]
async fn get_heatmap_data(app: tauri::AppHandle) -> String {
    let path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let database = database::Database::new(&path);
    let photos = database.get_all_photos_with_location();
    serde_json::to_string(&photos).unwrap()
}

#[tauri::command]
async fn generate_dummy_data(app: tauri::AppHandle) {
    use rand::Rng;
    let path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let database = database::Database::new(&path);
    let mut rng = rand::thread_rng();

    println!("Generating dummy data...");

    // Generate 500 dummy photos distributed around the world
    for i in 0..500 {
        let lat = rng.gen_range(-90.0..90.0);
        let lon = rng.gen_range(-180.0..180.0);

        let photo = database::Photo {
            id: format!("dummy_{}", i),
            location: format!("/tmp/dummy_{}.jpg", i),
            encoded: String::new(), // Empty encoded string for dummy
            objects: std::collections::HashMap::new(),
            properties: std::collections::HashMap::new(),
            latitude: lat,
            longitude: lon,
            favorite: false,
        };
        database.store_photo(photo);
    }
    println!("Dummy data generated.");
}

#[tauri::command]
async fn toggle_favorite(app: tauri::AppHandle, id: String) -> bool {
    let path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let database = database::Database::new(&path);
    database.toggle_favorite(&id)
}
