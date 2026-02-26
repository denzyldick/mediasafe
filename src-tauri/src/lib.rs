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

#[derive(serde::Serialize, Clone)]
struct DownloadProgress {
    model: String,
    downloaded: u64,
    total: Option<u64>,
}

#[tauri::command]
async fn download_models(
    app: tauri::AppHandle,
    models: Vec<String>,
    state: tauri::State<'_, ml::MlContext>,
) -> Result<(), String> {
    use tauri::Emitter;
    use tokio::io::AsyncWriteExt;

    let app_config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    let models_dir = app_config_dir.join("models");
    std::fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;

    let mut files_to_download = Vec::new();
    for model in &models {
        if model == "clip" {
            files_to_download.push(("clip", "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/vision_model.onnx?download=true", "clip-vit-base-patch32-visual.onnx"));
            files_to_download.push(("clip", "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/text_model.onnx?download=true", "clip-vit-base-patch32-text.onnx"));
            files_to_download.push(("clip", "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/tokenizer.json?download=true", "tokenizer.json"));
        } else if model == "ultraface" {
            files_to_download.push(("ultraface", "https://github.com/onnx/models/blob/main/validated/vision/body_analysis/ultraface/models/version-RFB-320.onnx?raw=true", "version-RFB-320.onnx"));
        }
    }

    for (model_name, url, filename) in files_to_download {
        let path = models_dir.join(filename);
        let mut response = reqwest::get(url).await.map_err(|e| e.to_string())?;
        let total_size = response.content_length();
        let mut file = tokio::fs::File::create(&path)
            .await
            .map_err(|e| e.to_string())?;
        let mut downloaded: u64 = 0;

        while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
            file.write_all(&chunk).await.map_err(|e| e.to_string())?;
            downloaded += chunk.len() as u64;

            let _ = app.emit(
                "download-progress",
                DownloadProgress {
                    model: model_name.to_string(),
                    downloaded,
                    total: total_size,
                },
            );
        }
    }

    // Trigger reload
    if let Ok(tx) = state.tx.lock() {
        let _ = tx.send("__RELOAD_MODELS__".to_string());
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let config_path = app
                .path()
                .app_config_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            let tx = ml::start_background_worker(&app.handle(), config_path);
            app.manage(ml::MlContext {
                tx: std::sync::Mutex::new(tx),
            });
            Ok(())
        })
        // .plugin(tauri_plugin_devtools::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            download_models,
            get_initial_state,
            list_files,
            scan_files,
            get_last_scan_time,
            get_ip,
            get_heatmap_data,
            generate_dummy_data,
            toggle_favorite,
            get_faces,
            server::generate_pairing_codes,
            server::hash_pairing_code,
            start_webrtc_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn join_network(_ip: String) -> String {
    // This function will be rewritten to use the new WebSocket signaling server
    // For now, return a placeholder
    "Connecting to signaling server...".to_string()
}

mod config;
mod database;
// Removed device.rs module
mod directory;
mod face_detector;
mod file;
mod ml;
mod server;
mod transport;

#[tauri::command]
async fn start_webrtc_session(
    room_id: String,
    is_initiator: bool,
    signaling_url: String,
) -> Result<(), String> {
    println!("Starting WebRTC session for room_id: {}", room_id);
    // Spawn WebRTC connection loop in the background so Tauri remains responsive
    tauri::async_runtime::spawn(async move {
        let client = transport::WebRtcClient {
            room_id,
            is_initiator,
            signaling_url,
        };
        if let Err(e) = client.start().await {
            println!("WebRTC signaling failed: {}", e);
        }
    });

    Ok(())
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

// Obsolete TCP list_devices command removed.
// Device discovery is now handled passively via passing the 4-word mnemonic or QR code.
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
fn scan_files(app: tauri::AppHandle) {
    std::thread::spawn(move || {
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

        let log_msg = format!(
            "Found {} directories to scan in config path: {}",
            total_dirs, path
        );
        println!("{}", log_msg);
        let _ = app.emit("log-message", log_msg);

        for (index, directory) in directories.iter().enumerate() {
            let msg = format!(
                "Scanning folder {} ({}/{})",
                directory,
                index + 1,
                total_dirs
            );
            println!("{}", msg);
            let _ = app.emit("log-message", msg);

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

            file::scan_folder(&app, directory.clone(), &path);
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
    });
}

#[tauri::command]
async fn get_thumbnail(path: String) -> String {
    println!("Generating thumnail for {}", path);
    file::get_thumbnail(path)
}

// Obsolete get_device_by_name command removed.

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
    let msg = format!(
        "Command add_directory called with path: {} and config_path: {}",
        path, config_path
    );
    println!("{}", msg);
    use tauri::Emitter;
    let _ = app.emit("log-message", msg);
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

#[tauri::command]
async fn get_faces(app: tauri::AppHandle) -> String {
    let path = app
        .path()
        .app_config_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let database = database::Database::new(&path);
    let faces = database.get_all_faces();
    serde_json::to_string(&faces).unwrap()
}
