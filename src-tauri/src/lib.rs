// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
use tauri::Manager;

fn get_config_path(app: &tauri::AppHandle) -> String {
    app.path().app_config_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|e| {
            println!("ERROR: Could not resolve config dir: {}", e);
            "".to_string()
        })
}

#[tauri::command]
async fn get_last_scan_time(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() { return "Never".to_string(); }
    let database = database::Database::new(&path);
    database.get_last_scan_time().unwrap_or_else(|| "Never".to_string())
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

    let path = get_config_path(&app);
    if path.is_empty() { return Err("Could not resolve config dir".to_string()); }
    let models_dir = std::path::PathBuf::from(&path).join("models");
    std::fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;

    let mut files_to_download: Vec<(String, String, String)> = Vec::new();
    for model in &models {
        let m = model.to_lowercase();
        if m == "clip" {
            files_to_download.push(("clip".to_string(), "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/vision_model.onnx?download=true".to_string(), "clip-vit-base-patch32-visual.onnx".to_string()));
            files_to_download.push(("clip".to_string(), "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/text_model.onnx?download=true".to_string(), "clip-vit-base-patch32-text.onnx".to_string()));
            files_to_download.push(("clip".to_string(), "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/tokenizer.json?download=true".to_string(), "tokenizer.json".to_string()));
        } else if m == "ultraface" {
            files_to_download.push(("ultraface".to_string(), "https://raw.githubusercontent.com/Linzaer/Ultra-Light-Fast-Generic-Face-Detector-1MB/master/models/onnx/version-RFB-320.onnx".to_string(), "version-RFB-320.onnx".to_string()));
        }
    }

    let tx = match state.tx.lock() {
        Ok(t) => t.clone(),
        Err(e) => return Err(e.to_string()),
    };

    tauri::async_runtime::spawn(async move {
        emit_log(&app, format!("Download sequence started. Queue size: {}", files_to_download.len()));
        if let Err(e) = std::fs::create_dir_all(&models_dir) {
            emit_log(&app, format!("ERROR: Could not create models directory: {}", e));
            return;
        }

        let client = match reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36")
            .timeout(std::time::Duration::from_secs(600))
            .connect_timeout(std::time::Duration::from_secs(60))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build() {
                Ok(c) => c,
                Err(e) => {
                    emit_log(&app, format!("ERROR: Failed to create HTTP client: {}", e));
                    return;
                }
            };

        for (model_name, url, filename) in files_to_download {
            let path = models_dir.join(&filename);
            emit_log(&app, format!("Initiating download: {}", filename));
            let mut response = match client.get(&url).send().await {
                Ok(r) => {
                    emit_log(&app, format!("Response received for {}: Status {}", filename, r.status()));
                    r
                },
                Err(e) => {
                    emit_log(&app, format!("ERROR: Request failed for {}: {}", filename, e));
                    continue;
                }
            };

            if !response.status().is_success() { continue; }
            let total_size = response.content_length();
            let mut file = match tokio::fs::File::create(&path).await {
                Ok(f) => f,
                Err(e) => {
                    emit_log(&app, format!("ERROR: Failed to create file {}: {}", filename, e));
                    continue;
                }
            };
            let mut downloaded: u64 = 0;
            while let Ok(Some(chunk)) = response.chunk().await {
                if let Err(_) = file.write_all(&chunk).await { break; }
                downloaded += chunk.len() as u64;
                let _ = app.emit("download-progress", DownloadProgress { model: model_name.clone(), downloaded, total: total_size });
            }
            emit_log(&app, format!("SUCCESS: Finished downloading {}", filename));
        }
        let _ = tx.send("__RELOAD_MODELS__".to_string());
        let _ = app.emit("download-complete", ());
    });
    Ok(())
}

#[tauri::command]
async fn check_models(app: tauri::AppHandle) -> Vec<String> {
    emit_log(&app, "Starting model check...".to_string());
    let path = get_config_path(&app);
    if path.is_empty() { return Vec::new(); }
    
    let models_dir = std::path::PathBuf::from(&path).join("models");
    emit_log(&app, format!("Checking models directory: {:?}", models_dir));
    
    if !models_dir.exists() {
        emit_log(&app, "ERROR: Models directory does not exist!".to_string());
        return Vec::new();
    }

    let mut downloaded = Vec::new();
    let clip_files = vec!["clip-vit-base-patch32-visual.onnx", "clip-vit-base-patch32-text.onnx", "tokenizer.json"];
    let mut clip_ok = true;
    for file in clip_files { 
        let p = models_dir.join(file);
        if !p.exists() {
            emit_log(&app, format!("Missing CLIP file: {}", file));
            clip_ok = false; 
            break; 
        } else {
            emit_log(&app, format!("Found CLIP file: {}", file));
        }
    }
    if clip_ok { downloaded.push("clip".to_string()); }
    
    let ultra_path = models_dir.join("version-RFB-320.onnx");
    if ultra_path.exists() { 
        emit_log(&app, "Found UltraFace model file.".to_string());
        downloaded.push("ultraface".to_string()); 
    } else {
        emit_log(&app, "Missing UltraFace model file.".to_string());
    }
    
    emit_log(&app, format!("Model check complete. Detected: {:?}", downloaded));
    downloaded
}

#[tauri::command]
async fn get_people(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() { return "[]".to_string(); }
    let database = database::Database::new(&path);
    serde_json::to_string(&database.get_people()).unwrap_or("[]".to_string())
}

#[tauri::command]
async fn get_unnamed_faces(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() { return "[]".to_string(); }
    let database = database::Database::new(&path);
    serde_json::to_string(&database.get_unnamed_faces()).unwrap_or("[]".to_string())
}

#[tauri::command]
async fn assign_name_to_face(app: tauri::AppHandle, face_id: String, name: String) -> String {
    let path = get_config_path(&app);
    if path.is_empty() { return "".to_string(); }
    let database = database::Database::new(&path);
    database.assign_name_to_face(&face_id, &name)
}

#[tauri::command]
async fn get_person_photos(app: tauri::AppHandle, person_id: String) -> String {
    let path = get_config_path(&app);
    if path.is_empty() { return "[]".to_string(); }
    let database = database::Database::new(&path);
    serde_json::to_string(&database.get_photos_for_person(&person_id)).unwrap_or("[]".to_string())
}

#[tauri::command]
async fn is_initialized(app: tauri::AppHandle) -> bool {
    let path = get_config_path(&app);
    if path.is_empty() { return false; }
    let database = database::Database::new(&path);
    !database.list_directories().is_empty()
}

#[tauri::command]
async fn get_top_tags(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() { return "[]".to_string(); }
    let database = database::Database::new(&path);
    let mut tags: Vec<String> = Vec::new();
    if let Ok(mut stmt) = database.connection.prepare("SELECT class FROM object GROUP BY class ORDER BY COUNT(*) DESC LIMIT 5") {
        if let Ok(iter) = stmt.query_map([], |row| row.get(0)) {
            for item in iter { if let Ok(s) = item { tags.push(s); } }
        }
    }
    serde_json::to_string(&tags).unwrap_or("[]".to_string())
}

#[tauri::command]
async fn merge_people(app: tauri::AppHandle, from_id: String, to_id: String) {
    let path = get_config_path(&app);
    if path.is_empty() { return; }
    let db = database::Database::new(&path);
    db.merge_people(&from_id, &to_id);
}

#[tauri::command]
async fn rename_person(app: tauri::AppHandle, id: String, new_name: String) {
    let path = get_config_path(&app);
    if path.is_empty() { return; }
    let db = database::Database::new(&path);
    db.rename_person(&id, &new_name);
}

#[tauri::command]
async fn cleanup_database(app: tauri::AppHandle) {
    let path = get_config_path(&app);
    if path.is_empty() { return; }
    let db = database::Database::new(&path);
    let _ = db.connection.execute("VACUUM", ());
}

#[tauri::command]
async fn remove_directory_full(app: tauri::AppHandle, path: String) {
    let config_path = get_config_path(&app);
    if config_path.is_empty() { return; }
    let db = database::Database::new(&config_path);
    db.remove_directory_full(&path);
}

#[tauri::command]
async fn start_webrtc_session(
    app: tauri::AppHandle,
    room_id: String,
    is_initiator: bool,
    signaling_url: String,
) -> Result<(), String> {
    println!("Starting WebRTC session for room_id: {}", room_id);
    let app_handle = app.clone();
    let config_path = get_config_path(&app);
    if config_path.is_empty() { return Err("Could not resolve config dir".to_string()); }

    // Spawn WebRTC connection loop in the background so Tauri remains responsive
    tauri::async_runtime::spawn(async move {
        let client = transport::WebRtcClient {
            room_id,
            is_initiator,
            signaling_url,
            app_handle: Some(app_handle),
            config_path,
        };
        if let Err(e) = client.start().await {
            println!("WebRTC signaling failed: {}", e);
        }
    });

    Ok(())
}

#[tauri::command]
async fn get_indexing_status(state: tauri::State<'_, ml::MlContext>) -> Result<usize, String> {
    Ok(state.pending_count.load(std::sync::atomic::Ordering::SeqCst))
}

#[tauri::command]
async fn update_video_thumbnail(app: tauri::AppHandle, id: String, encoded: String) {
    let path = get_config_path(&app);
    if path.is_empty() { return; }
    let database = database::Database::new(&path);
    database.update_photo_thumbnail(&id, &encoded);
    use tauri::Emitter;
    let _ = app.emit("video-thumbnail-updated", serde_json::json!({ "id": id, "encoded": encoded }));
}

#[tauri::command]
async fn process_video_frames(id: String, frames: Vec<String>, state: tauri::State<'_, ml::MlContext>) -> Result<(), String> {
    if let Ok(tx) = state.tx.lock() {
        let payload = format!("__VIDEO_FRAMES__:{id}|||{}", frames.join("|||"));
        state.pending_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let _ = tx.send(payload);
    }
    Ok(())
}

#[tauri::command]
async fn clear_logs(app: tauri::AppHandle) {
    let path = get_config_path(&app);
    if path.is_empty() { return; }
    let db = database::Database::new(&path);
    db.clear_logs();
}

#[tauri::command]
async fn get_logs(app: tauri::AppHandle, limit: usize) -> String {
    let path = get_config_path(&app);
    if path.is_empty() { return "[]".to_string(); }
    let db = database::Database::new(&path);
    serde_json::to_string(&db.get_logs(limit)).unwrap_or("[]".to_string())
}

pub fn emit_log(app: &tauri::AppHandle, message: String) {
    use tauri::Emitter;
    let _ = app.emit("log-message", &message);
    let path = get_config_path(app);
    if !path.is_empty() {
        let db = database::Database::new(&path);
        let level = if message.to_lowercase().contains("error") { "error" } else { "info" };
        db.store_log(level, &message);
    }
}

#[tauri::command]
async fn get_config_dir(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    println!("Frontend requested config dir: {}", path);
    path
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let num_threads = if cfg!(any(target_os = "android", target_os = "ios")) { 2 } else { num_cpus::get() };
    let _ = rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global();

    tauri::Builder::default()
        .setup(|app| {
            let config_path = app.path().app_config_dir().map_err(|e| {
                println!("CRITICAL ERROR: Could not resolve config dir: {}", e);
                e
            })?;
            println!("Backend initializing at: {:?}", config_path);
            let config_path_str = config_path.to_str().unwrap().to_string();
            let (tx, pending_count) = ml::start_background_worker(&app.handle(), config_path_str);
            app.manage(ml::MlContext { tx: std::sync::Mutex::new(tx), pending_count });
            Ok(())
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_logs, clear_logs,
            get_config_dir,
            get_top_tags, get_indexing_status, check_models, is_initialized, get_people, get_unnamed_faces,
            assign_name_to_face, get_person_photos, download_models, get_initial_state, set_initial_state,
            list_files, scan_files, get_last_scan_time, get_os, get_ip, get_heatmap_data, save_config,
            get_config, generate_dummy_data, toggle_favorite, get_faces, get_raw_photo, list_directories,
            add_directory, remove_directory, get_thumbnail, list_objects, join_network,
            server::generate_pairing_codes, server::hash_pairing_code, start_webrtc_session,
            update_video_thumbnail, process_video_frames, merge_people, rename_person, cleanup_database,
            remove_directory_full
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn get_os() -> String { std::env::consts::OS.to_string() }

#[tauri::command]
async fn save_config(app: tauri::AppHandle, key: String, value: String) {
    let path = get_config_path(&app);
    if path.is_empty() { return; }
    let database = database::Database::new(&path);
    let mut map = std::collections::HashMap::new();
    map.insert(key, value);
    database.set_state(map);
}

#[tauri::command]
async fn get_config(app: tauri::AppHandle, key: String) -> Option<String> {
    let path = get_config_path(&app);
    if path.is_empty() { return None; }
    let database = database::Database::new(&path);
    database.get_state().get(&key).cloned()
}

#[tauri::command]
async fn join_network(_ip: String) -> String { "Connecting to signaling server...".to_string() }

mod config;
mod database;
mod directory;
mod face_detector;
mod file;
mod ml;
mod server;
pub mod transport;

use get_if_addrs::get_if_addrs;
use serde_json::from_str;
use std::net::Ipv4Addr;
use std::collections::HashMap;

#[tauri::command()]
async fn get_ip() -> String {
    let ifaces = get_if_addrs().unwrap();
    let mut ip = String::new();
    for iface in ifaces {
        if let std::net::IpAddr::V4(ipv4) = iface.ip() {
            if is_local_network_ip(ipv4) { ip = ipv4.to_string(); break; }
        }
    }
    ip
}

fn is_local_network_ip(ip: Ipv4Addr) -> bool {
    ip.octets()[0] == 10 || (ip.octets()[0] == 172 && (16..=31).contains(&ip.octets()[1])) || (ip.octets()[0] == 192 && ip.octets()[1] == 168)
}

#[tauri::command]
async fn list_objects(app: tauri::AppHandle, query: String) -> String {
    let path = get_config_path(&app);
    if path.is_empty() { return "[]".to_string(); }
    let database = database::Database::new(&path);
    serde_json::to_string(&database.list_objects(&query)).unwrap_or("[]".to_string())
}

use serde::{Deserialize, Serialize};
use std::string::String;

#[derive(Serialize, Deserialize, Debug)]
struct Image { path: String, encoded: String }

#[tauri::command]
async fn list_files(app: tauri::AppHandle, query: String, limit: usize, offset: usize, _scan: bool, favorites_only: bool) -> String {
    let path = get_config_path(&app);
    if path.is_empty() { return "[]".to_string(); }
    let database = database::Database::new(&path);
    let photos = database.list_photos(&query, offset, limit, favorites_only);
    serde_json::to_string(&photos).unwrap_or("[]".to_string())
}

#[tauri::command]
fn scan_files(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let path = get_config_path(&app);
        if path.is_empty() { return; }
        use std::time::SystemTime;
        use tauri::Emitter;
        let directories = directory::list_directories(&path);
        let total_dirs = directories.len();
        for (index, directory) in directories.iter().enumerate() {
            let _ = app.emit("scan-progress", serde_json::json!({ "status": "scanning", "current_directory": directory, "progress": ((index + 1) as f32 / total_dirs as f32 * 100.0) as u32, "current": index + 1, "total": total_dirs }));
            file::scan_folder(&app, directory.clone(), &path);
        }
        let database = database::Database::new(&path);
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs().to_string();
        database.set_last_scan_time(timestamp);
        let _ = app.emit("scan-progress", serde_json::json!({ "status": "complete", "progress": 100 }));
    });
}

#[tauri::command]
async fn get_raw_photo(path: String) -> String { file::read_file_base64(path) }

#[tauri::command]
async fn get_thumbnail(path: String) -> String { file::get_thumbnail(path) }

#[tauri::command]
async fn list_directories(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() { return "[]".to_string(); }
    serde_json::to_string(&directory::list_directories(&path)).unwrap_or("[]".to_string())
}

#[tauri::command]
async fn remove_directory(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let config_path = app.path().app_config_dir().map_err(|e| e.to_string())?.to_str().ok_or("Invalid config path")?.to_string();
    directory::remove_directory(path, &config_path);
    Ok(())
}

#[tauri::command]
async fn add_directory(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let config_path = app.path().app_config_dir().map_err(|e| e.to_string())?.to_str().ok_or("Invalid config path")?.to_string();
    directory::add_directory(path, &config_path);
    Ok(())
}

#[tauri::command]
async fn get_initial_state(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() { return "{}".to_string(); }
    let database = database::Database::new(&path);
    serde_json::to_string(&database.get_state()).unwrap_or("{}".to_string())
}

#[tauri::command]
async fn set_initial_state(app: tauri::AppHandle, state: String) {
    let state = match from_str::<HashMap<String, String>>(&state) { Ok(s) => s, Err(_) => return };
    let path = get_config_path(&app);
    if path.is_empty() { return; }
    let database = database::Database::new(&path);
    database.set_state(state);
}

#[tauri::command]
async fn get_heatmap_data(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() { return "[]".to_string(); }
    let database = database::Database::new(&path);
    serde_json::to_string(&database.get_all_photos_with_location()).unwrap_or("[]".to_string())
}

#[tauri::command]
async fn generate_dummy_data(app: tauri::AppHandle) {
    let path = get_config_path(&app);
    if path.is_empty() { return; }
    let database = database::Database::new(&path);
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for i in 0..500 {
        let photo = database::Photo {
            id: format!("dummy_{}", i), location: format!("/tmp/dummy_{}.jpg", i), encoded: String::new(),
            created: "2026:01:01 12:00:00".to_string(), objects: std::collections::HashMap::new(),
            properties: std::collections::HashMap::new(), latitude: rng.gen_range(-90.0..90.0),
            longitude: rng.gen_range(-180.0..180.0), favorite: false,
        };
        database.store_photo(photo);
    }
}

#[tauri::command]
async fn toggle_favorite(app: tauri::AppHandle, id: String) -> bool {
    let path = get_config_path(&app);
    if path.is_empty() { return false; }
    let database = database::Database::new(&path);
    database.toggle_favorite(&id)
}

#[tauri::command]
async fn get_faces(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() { return "[]".to_string(); }
    let database = database::Database::new(&path);
    serde_json::to_string(&database.get_all_faces()).unwrap_or("[]".to_string())
}
