use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
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
    let state = app.state::<ml::MlContext>();
    state
        .abort
        .store(false, std::sync::atomic::Ordering::SeqCst);

    let abort_flag = Arc::clone(&state.abort);

    std::thread::spawn(move || {
        let total = folders.len();
        for (i, folder) in folders.iter().enumerate() {
            if abort_flag.load(std::sync::atomic::Ordering::SeqCst) {
                return;
            }
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
async fn check_models(app: tauri::AppHandle) -> Vec<String> {
    let path = get_config_path(&app);
    let mut downloaded = Vec::new();
    if path.is_empty() {
        return downloaded;
    }
    let models_dir = Path::new(&path).join("models");

    let clip_files = [
        "clip-vit-base-patch32-visual.onnx",
        "clip-vit-base-patch32-text.onnx",
        "tokenizer.json",
    ];
    let mut clip_ok = true;
    for name in clip_files {
        let p = models_dir.join(name);
        if !p.exists()
            || (name != "tokenizer.json"
                && p.metadata().map(|m| m.len()).unwrap_or(0) < 1024 * 1024)
        {
            clip_ok = false;
            break;
        }
    }
    if clip_ok {
        downloaded.push("clip".to_string());
    }

    let ultraface_path = models_dir.join("version-RFB-320.onnx");
    if ultraface_path.exists()
        && ultraface_path.metadata().map(|m| m.len()).unwrap_or(0) > 1024 * 1024
    {
        downloaded.push("ultraface".to_string());
    }

    downloaded
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
    use tokio::io::AsyncWriteExt;

    let path = get_config_path(&app);
    if path.is_empty() {
        return Err("Could not resolve config dir".to_string());
    }
    let models_dir = std::path::PathBuf::from(&path).join("models");
    std::fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;

    let mut files_to_download: Vec<(String, String, String)> = Vec::new();
    for model in &models {
        let m = model.to_lowercase();
        if m == "clip" {
            files_to_download.push(("clip-visual".to_string(), "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/vision_model.onnx?download=true".to_string(), "clip-vit-base-patch32-visual.onnx".to_string()));
            files_to_download.push(("clip-text".to_string(), "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/text_model.onnx?download=true".to_string(), "clip-vit-base-patch32-text.onnx".to_string()));
            files_to_download.push(("clip-tokenizer".to_string(), "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/tokenizer.json?download=true".to_string(), "tokenizer.json".to_string()));
        } else if m == "ultraface" {
            files_to_download.push(("ultraface".to_string(), "https://raw.githubusercontent.com/Linzaer/Ultra-Light-Fast-Generic-Face-Detector-1MB/master/models/onnx/version-RFB-320.onnx".to_string(), "version-RFB-320.onnx".to_string()));
        }
    }

    let tx = match state.tx.lock() {
        Ok(t) => t.clone(),
        Err(e) => return Err(e.to_string()),
    };

    tauri::async_runtime::spawn(async move {
        emit_log(
            &app,
            format!(
                "Download sequence started. Queue size: {}",
                files_to_download.len()
            ),
        );

        let client = match reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36")
            .timeout(std::time::Duration::from_secs(600))
            .connect_timeout(std::time::Duration::from_secs(60))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build() {
                Ok(c) => c,
                Err(e) => {
                    emit_log(&app, format!("ERROR: Failed to create HTTP client: {e}"));
                    return;
                }
            };

        for (model_name, url, filename) in files_to_download {
            let path = models_dir.join(&filename);
            emit_log(&app, format!("Initiating download: {filename}"));
            let mut response = match client.get(&url).send().await {
                Ok(r) => {
                    emit_log(
                        &app,
                        format!("Response received for {}: Status {}", filename, r.status()),
                    );
                    r
                }
                Err(e) => {
                    emit_log(&app, format!("ERROR: Request failed for {filename}: {e}"));
                    continue;
                }
            };

            if !response.status().is_success() {
                emit_log(
                    &app,
                    format!(
                        "ERROR: Download failed for {filename}: Status {}",
                        response.status()
                    ),
                );
                continue;
            }
            let total_size = response.content_length();
            let tmp_path = path.with_extension("tmp");
            let mut file = match tokio::fs::File::create(&tmp_path).await {
                Ok(f) => f,
                Err(e) => {
                    emit_log(
                        &app,
                        format!("ERROR: Failed to create temp file {filename}: {e}"),
                    );
                    continue;
                }
            };
            let mut downloaded: u64 = 0;
            let mut success = true;
            while let Ok(Some(chunk)) = response.chunk().await {
                if (file.write_all(&chunk).await).is_err() {
                    success = false;
                    break;
                }
                downloaded += chunk.len() as u64;
                let _ = app.emit(
                    "download-progress",
                    DownloadProgress {
                        model: model_name.clone(),
                        downloaded,
                        total: total_size,
                    },
                );
            }

            if success {
                drop(file);
                if let Err(e) = tokio::fs::rename(&tmp_path, &path).await {
                    emit_log(&app, format!("ERROR: Failed to move {filename}: {e}"));
                    let _ = tokio::fs::remove_file(&tmp_path).await;
                } else {
                    emit_log(&app, format!("SUCCESS: Finished downloading {filename}"));
                }
            } else {
                let _ = tokio::fs::remove_file(&tmp_path).await;
                emit_log(&app, format!("ERROR: Download interrupted for {filename}"));
            }
        }
        let _ = tx.send("__RELOAD_MODELS__".to_string());
        let _ = app.emit("download-complete", ());
    });

    Ok(())
}

#[tauri::command]
async fn list_files(
    app: tauri::AppHandle,
    offset: usize,
    limit: usize,
    query: String,
    scan: bool,
    favoritesOnly: bool,
    videosOnly: bool,
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
        favoritesOnly,
        videosOnly,
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
async fn get_raw_photo(path: String) -> String {
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
async fn get_person_faces(app: tauri::AppHandle, person_id: String) -> String {
    let path = get_config_path(&app);
    if path.is_empty() {
        return "[]".to_string();
    }
    let database = database::Database::new(&path);
    serde_json::to_string(&database.get_person_faces(&person_id)).unwrap_or("[]".to_string())
}

#[tauri::command]
async fn get_faces_for_photo(app: tauri::AppHandle, photo_id: String) -> String {
    let path = get_config_path(&app);
    if path.is_empty() {
        return "[]".to_string();
    }
    let database = database::Database::new(&path);
    serde_json::to_string(&database.get_faces_for_photo(&photo_id)).unwrap_or("[]".to_string())
}

#[tauri::command]
async fn delete_face(app: tauri::AppHandle, face_id: String) {
    let path = get_config_path(&app);
    if path.is_empty() {
        return;
    }
    let database = database::Database::new(&path);
    let _ = database
        .connection
        .execute("DELETE FROM faces WHERE face_id = ?1", [&face_id]);
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
    let db_path = std::path::Path::new(&path).join("siegu.db");
    if db_path.exists() {
        let _ = std::fs::remove_file(db_path);
    }
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
    println!("Adding new device: {name} at {ip}");
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
    let mut devices = db.list_devices();

    // Add current host
    let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string());
    let (photo_count, video_count) = db.get_media_counts();

    devices.insert(
        0,
        database::DeviceInfo {
            id: "host".to_string(),
            title: format!("Siegu ({hostname})"),
            icon: "mdi-laptop".to_string(),
            up_to_date: true,
            host: true,
            photo_count,
            video_count,
            os: std::env::consts::OS.to_string(),
        },
    );

    serde_json::to_string(&devices).unwrap_or("[]".to_string())
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
    _app: tauri::AppHandle,
    state: tauri::State<'_, ml::MlContext>,
    id: String,
    frames: Vec<String>,
) {
    let mut payload = format!("__VIDEO_FRAMES__:{id}");
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

    // Force indexing_mode to immediate so the worker actually processes the items
    let mut state_map = std::collections::HashMap::new();
    state_map.insert("indexing_mode".to_string(), "immediate".to_string());
    db.set_state(state_map);

    let mut photo_ids = Vec::new();
    if let Ok(mut stmt) = db.connection.prepare("SELECT id FROM photo") {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for id in rows.flatten() {
                photo_ids.push(id);
            }
        }
    }
    println!("Found {} photos to index", photo_ids.len());
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
async fn abort_indexing(state: tauri::State<'_, ml::MlContext>) -> Result<(), String> {
    if let Ok(tx) = state.tx.lock() {
        let _ = tx.send("__ABORT__".to_string());
    }
    state.abort.store(true, std::sync::atomic::Ordering::SeqCst);
    state
        .pending_count
        .store(0, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
async fn get_heatmap_data(app: tauri::AppHandle) -> String {
    let path = get_config_path(&app);
    if path.is_empty() {
        return "[]".to_string();
    }
    let database = database::Database::new(&path);
    let photos = database.get_all_photos_with_location();
    println!("DEBUG: Found {} photos with GPS for heatmap", photos.len());
    serde_json::to_string(&photos).unwrap_or("[]".to_string())
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

#[tauri::command]
async fn get_logs(app: tauri::AppHandle, limit: usize) -> String {
    let path = get_config_path(&app);
    if path.is_empty() {
        return "[]".to_string();
    }
    let database = database::Database::new(&path);
    serde_json::to_string(&database.get_logs(limit)).unwrap_or("[]".to_string())
}

#[tauri::command]
async fn clear_logs(app: tauri::AppHandle) {
    let path = get_config_path(&app);
    if path.is_empty() {
        return;
    }
    let database = database::Database::new(&path);
    database.clear_logs();
}

pub fn emit_log(app: &tauri::AppHandle, message: String) {
    println!("{message}");
    let _ = app.emit("log-message", message.clone());
    let path = get_config_path(app);
    if !path.is_empty() {
        let database = database::Database::new(&path);
        let level = if message.to_lowercase().contains("error") {
            "error"
        } else {
            "info"
        };
        database.store_log(level, &message);
    }
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
            let (tx, pending_count, abort) =
                ml::start_background_worker(app.handle(), config_path.clone());
            app.manage(ml::MlContext {
                tx: std::sync::Mutex::new(tx),
                pending_count,
                abort,
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
            download_models,
            get_logs,
            clear_logs,
            list_files,
            get_thumbnail,
            get_last_scan_time,
            toggle_favorite,
            add_directory,
            list_directories,
            remove_directory,
            read_file_base64,
            get_raw_photo,
            get_people,
            get_unnamed_faces,
            assign_name_to_face,
            get_person_photos,
            rename_person,
            merge_people,
            is_initialized,
            get_top_tags,
            get_person_faces,
            get_faces_for_photo,
            delete_face,
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
            abort_indexing,
            get_os,
            save_config,
            get_config,
            get_indexing_status,
            get_heatmap_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
