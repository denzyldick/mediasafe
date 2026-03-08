use serde_json;
use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;
use tauri::Emitter;
use tauri::Manager;

mod config;
mod database;
mod directory;
mod face_detector;
mod file;
mod ml;
mod server;
mod transport;

struct WebRtcState {
    active_session: std::sync::Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
}

fn get_config_path(app: &tauri::AppHandle) -> String {
    app.path()
        .app_config_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "".to_string())
}

#[tauri::command]
fn scan_files(app: tauri::AppHandle) {
    let path = get_config_path(&app);
    if path.is_empty() {
        return;
    }
    let database = database::Database::new(&path);
    let folders = database.list_directories();

    std::thread::spawn(move || {
        let total = folders.len();
        for (i, folder) in folders.iter().enumerate() {
            let progress = (i as f32 / total as f32 * 100.0) as u32;
            let _ = app.emit("scan-progress", serde_json::json!({ "status": "scanning", "progress": progress, "current": i + 1, "total": total, "current_directory": folder }));
            file::scan_folder(&app, folder.clone(), &path);
        }
        let database = database::Database::new(&path);
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();
        database.set_last_scan_time(timestamp);
        let _ = app.emit(
            "scan-progress",
            serde_json::json!({ "status": "complete", "progress": 100 }),
        );
    });
}

#[tauri::command]
async fn check_models(app: tauri::AppHandle) -> bool {
    let path = get_config_path(&app);
    if path.is_empty() {
        return false;
    }
    let models_dir = Path::new(&path).join("models");
    let required = [
        "clip-vit-base-patch32-visual.onnx",
        "clip-vit-base-patch32-text.onnx",
        "tokenizer.json",
        "version-RFB-320.onnx",
    ];
    for name in required {
        let p = models_dir.join(name);
        if !p.exists() || p.metadata().map(|m| m.len()).unwrap_or(0) < 1024 * 1024 {
            if name != "tokenizer.json" {
                return false;
            }
        }
    }
    true
}

#[tauri::command]
async fn list_files(
    app: tauri::AppHandle,
    offset: usize,
    limit: usize,
    query: String,
    scan: bool,
    favorites_only: bool,
    videos_only: bool,
) -> Result<String, String> {
    let path = get_config_path(&app);
    if path.is_empty() {
        return Ok("[]".to_string());
    }
    if scan {
        scan_files(app.clone());
    }
    let database = database::Database::new(&path);
    Ok(serde_json::to_string(&database.list_photos(
        &query,
        offset,
        limit,
        favorites_only,
        videos_only,
    ))
    .unwrap_or("[]".to_string()))
}

#[tauri::command]
async fn get_thumbnail(_app: tauri::AppHandle, path: String) -> String {
    file::get_thumbnail(path)
}

#[tauri::command]
async fn get_last_scan_time(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() {
        return "Never".to_string();
    }
    let database = database::Database::new(&path);
    database.get_last_scan_time().unwrap_or("Never".to_string())
}

#[tauri::command]
async fn toggle_favorite(app: tauri::AppHandle, id: String) -> bool {
    let path = get_config_path(&app);
    if path.is_empty() {
        return false;
    }
    let database = database::Database::new(&path);
    database.toggle_favorite(&id)
}

#[tauri::command]
async fn add_directory(app: tauri::AppHandle, path: String) {
    let config_path = get_config_path(&app);
    if config_path.is_empty() {
        return;
    }
    let database = database::Database::new(&config_path);
    database.add_directory(&path);
}

#[tauri::command]
async fn list_directories(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() {
        return "[]".to_string();
    }
    let database = database::Database::new(&path);
    serde_json::to_string(&database.list_directories()).unwrap_or("[]".to_string())
}

#[tauri::command]
async fn remove_directory(app: tauri::AppHandle, path: String) {
    let config_path = get_config_path(&app);
    if config_path.is_empty() {
        return;
    }
    let database = database::Database::new(&config_path);
    database.remove_directory(path);
}

#[tauri::command]
async fn read_file_base64(path: String) -> String {
    file::read_file_base64(path)
}

#[tauri::command]
async fn get_people(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() {
        return "[]".to_string();
    }
    let database = database::Database::new(&path);
    serde_json::to_string(&database.get_people()).unwrap_or("[]".to_string())
}

#[tauri::command]
async fn get_unnamed_faces(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() {
        return "[]".to_string();
    }
    let database = database::Database::new(&path);
    serde_json::to_string(&database.get_anonymous_people_groups()).unwrap_or("[]".to_string())
}

#[tauri::command]
fn assign_name_to_face(
    app: tauri::AppHandle,
    state: tauri::State<'_, ml::MlContext>,
    face_id: String,
    name: String,
) -> String {
    let path = get_config_path(&app);
    if path.is_empty() {
        return "".to_string();
    }
    let database = database::Database::new(&path);
    let id = database.assign_name_to_face(&face_id, &name);

    if let Ok(tx) = state.tx.lock() {
        let _ = tx.send("__RELOAD_MODELS__".to_string());
    }
    id
}

#[tauri::command]
async fn get_person_photos(app: tauri::AppHandle, person_id: String) -> Result<String, String> {
    let path = get_config_path(&app);
    if path.is_empty() {
        return Ok("[]".to_string());
    }
    let database = database::Database::new(&path);
    Ok(
        serde_json::to_string(&database.get_photos_for_person(&person_id))
            .unwrap_or("[]".to_string()),
    )
}

#[tauri::command]
async fn is_initialized(app: tauri::AppHandle) -> bool {
    let path = get_config_path(&app);
    if path.is_empty() {
        return false;
    }
    let database = database::Database::new(&path);
    !database.list_directories().is_empty()
}

#[tauri::command]
async fn get_top_tags(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() {
        return "[]".to_string();
    }
    let database = database::Database::new(&path);
    let mut tags: Vec<String> = Vec::new();
    if let Ok(mut stmt) = database
        .connection
        .prepare("SELECT class FROM object GROUP BY class ORDER BY COUNT(*) DESC LIMIT 5")
    {
        if let Ok(iter) = stmt.query_map([], |row| row.get(0)) {
            for item in iter.flatten() {
                tags.push(item);
            }
        }
    }
    serde_json::to_string(&tags).unwrap_or("[]".to_string())
}

#[tauri::command]
fn merge_people(
    app: tauri::AppHandle,
    state: tauri::State<'_, ml::MlContext>,
    from_id: String,
    to_id: String,
) {
    let path = get_config_path(&app);
    if path.is_empty() {
        return;
    }
    let db = database::Database::new(&path);
    db.merge_people(&from_id, &to_id);

    if let Ok(tx) = state.tx.lock() {
        let _ = tx.send("__RELOAD_MODELS__".to_string());
    }
}

#[tauri::command]
async fn rename_person(app: tauri::AppHandle, id: String, new_name: String) {
    let path = get_config_path(&app);
    if path.is_empty() {
        return;
    }
    let db = database::Database::new(&path);
    db.rename_person(&id, &new_name);
}

#[tauri::command]
async fn cleanup_database(app: tauri::AppHandle) {
    let path = get_config_path(&app);
    if path.is_empty() {
        return;
    }
    let db = database::Database::new(&path);
    let _ = db.connection.execute("VACUUM", ());
}

#[tauri::command]
async fn remove_directory_full(app: tauri::AppHandle, path: String) {
    let config_path = get_config_path(&app);
    if config_path.is_empty() {
        return;
    }
    let db = database::Database::new(&config_path);
    db.remove_directory_full(&path);
}

#[tauri::command]
async fn start_webrtc_session(
    app: tauri::AppHandle,
    state: tauri::State<'_, WebRtcState>,
    roomId: String,
    isInitiator: bool,
    signalingUrl: String,
) -> Result<(), String> {
    let app_handle = app.clone();
    let config_path = get_config_path(&app);
    if config_path.is_empty() {
        return Err("Config error".to_string());
    }

    // Abort existing session if any
    if let Ok(mut session) = state.active_session.lock() {
        if let Some(handle) = session.take() {
            println!("Aborting previous WebRTC session");
            handle.abort();
        }

        let handle = tauri::async_runtime::spawn(async move {
            let client = transport::WebRtcClient {
                room_id: roomId,
                is_initiator: isInitiator,
                signaling_url: signalingUrl,
                app_handle: Some(app_handle),
                config_path,
            };
            let _ = client.start().await;
        });

        *session = Some(handle);
    }

    Ok(())
}

#[tauri::command]
fn get_indexing_status(state: tauri::State<'_, ml::MlContext>) -> usize {
    state
        .pending_count
        .load(std::sync::atomic::Ordering::SeqCst)
}

#[tauri::command]
async fn join_network(app: tauri::AppHandle, ip: String, name: String) {
    println!("Adding new device: {} at {}", name, ip);
    let path = get_config_path(&app);
    if path.is_empty() {
        return;
    }
    let db = database::Database::new(&path);
    let _ = db.connection.execute(
        "INSERT OR REPLACE INTO device(ip, name) VALUES(?1, ?2)",
        (ip, name),
    );
}

#[tauri::command]
async fn list_devices(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() {
        return "[]".to_string();
    }
    let db = database::Database::new(&path);
    serde_json::to_string(&db.list_devices()).unwrap_or("[]".to_string())
}

#[tauri::command]
async fn list_objects(app: tauri::AppHandle, query: String) -> Result<String, String> {
    let path = get_config_path(&app);
    if path.is_empty() {
        return Ok("[]".to_string());
    }
    let db = database::Database::new(&path);
    Ok(serde_json::to_string(&db.list_objects(&query)).unwrap_or("[]".to_string()))
}

#[tauri::command]
async fn update_video_thumbnail(app: tauri::AppHandle, id: String, b64: String) {
    let path = get_config_path(&app);
    if path.is_empty() {
        return;
    }
    let db = database::Database::new(&path);
    db.update_photo_thumbnail(&id, &b64);
}

#[tauri::command]
fn process_video_frames(
    app: tauri::AppHandle,
    state: tauri::State<'_, ml::MlContext>,
    id: String,
    frames: Vec<String>,
) {
    let mut payload = format!("__VIDEO_FRAMES__:{}", id);
    for frame in frames {
        payload.push_str("|||");
        payload.push_str(&frame);
    }
    if let Ok(tx) = state.tx.lock() {
        state
            .pending_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let _ = tx.send(payload);
    }
}

#[tauri::command]
async fn get_media_server_port(app: tauri::AppHandle) -> u16 {
    app.state::<transport::MediaServerState>().port
}

#[tauri::command]
async fn index_faces(
    app: tauri::AppHandle,
    state: tauri::State<'_, ml::MlContext>,
) -> Result<(), String> {
    println!("Face indexing requested...");
    let path = get_config_path(&app);
    if path.is_empty() {
        return Err("Config error".to_string());
    }
    let db = database::Database::new(&path);
    let mut photo_ids = Vec::new();
    if let Ok(mut stmt) = db.connection.prepare("SELECT id FROM photo") {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for id in rows.flatten() {
                photo_ids.push(id);
            }
        }
    }
    if let Ok(tx) = state.tx.lock() {
        let count = photo_ids.len();
        let total = state
            .pending_count
            .fetch_add(count, std::sync::atomic::Ordering::SeqCst)
            + count;
        let _ = app.emit("indexing-progress", total);
        for id in photo_ids {
            let _ = tx.send(id);
        }
    }
    Ok(())
}

#[tauri::command]
async fn get_heatmap_data(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() {
        return "[]".to_string();
    }
    let database = database::Database::new(&path);
    serde_json::to_string(&database.get_all_photos_with_location()).unwrap_or("[]".to_string())
}

#[tauri::command]
async fn get_os() -> String {
    std::env::consts::OS.to_string()
}

#[tauri::command]
async fn save_config(app: tauri::AppHandle, key: String, value: String) {
    let path = get_config_path(&app);
    if path.is_empty() {
        return;
    }
    let db = database::Database::new(&path);
    let mut state = HashMap::new();
    state.insert(key, value);
    db.set_state(state);
}

#[tauri::command]
async fn get_config(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() {
        return "{}".to_string();
    }
    let db = database::Database::new(&path);
    serde_json::to_string(&db.get_state()).unwrap_or("{}".to_string())
}

pub fn emit_log(app: &tauri::AppHandle, message: String) {
    println!("{}", message);
    let _ = app.emit("log-message", message);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config_path = get_config_path(app.handle());
            let (tx, pending_count) =
                ml::start_background_worker(app.handle(), config_path.clone());
            app.manage(ml::MlContext {
                tx: std::sync::Mutex::new(tx),
                pending_count,
            });

            let media_server_port = transport::start_media_server(config_path);
            app.manage(transport::MediaServerState {
                port: media_server_port,
            });

            app.manage(WebRtcState {
                active_session: std::sync::Mutex::new(None),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_files,
            check_models,
            list_files,
            get_thumbnail,
            get_last_scan_time,
            toggle_favorite,
            add_directory,
            list_directories,
            remove_directory,
            read_file_base64,
            get_people,
            get_unnamed_faces,
            assign_name_to_face,
            get_person_photos,
            is_initialized,
            get_top_tags,
            join_network,
            list_devices,
            server::generate_pairing_codes,
            server::hash_pairing_code,
            start_webrtc_session,
            update_video_thumbnail,
            process_video_frames,
            merge_people,
            rename_person,
            cleanup_database,
            remove_directory_full,
            get_media_server_port,
            index_faces,
            get_os,
            save_config,
            get_config,
            get_indexing_status,
            get_heatmap_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
