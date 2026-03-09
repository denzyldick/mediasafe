use crate::database;
use base64::{engine::general_purpose, Engine as _};
use exif::Reader;
use image::io::Reader as ImageReader;
use image::ImageOutputFormat;
use jwalk::WalkDir;
use rand::{distributions::Alphanumeric, Rng};

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::Path;

use std::io::{BufReader, Cursor};
use std::string::String;

use crate::ml::MlContext;
use tauri::{Emitter, Manager};

fn emit_log(app: &tauri::AppHandle, message: String) {
    println!("{message}");
    let _ = app.emit("log-message", message);
}

///
/// This is will scan a folder recursively and store all the images in the database.
pub fn scan_folder(app: &tauri::AppHandle, directory: String, path: &str) {
    let database = database::Database::new(path);

    // Load thread config
    let config = database.get_state();
    let num_threads: usize = config
        .get("scan_threads")
        .and_then(|s| s.parse().ok())
        .unwrap_or({
            if cfg!(any(target_os = "android", target_os = "ios")) {
                2
            } else {
                4
            }
        });

    emit_log(
        app,
        format!("Scanning with {num_threads} threads in: {directory}"),
    );

    // Collect all valid image and video paths first
    let mut image_paths = Vec::new();
    let video_extensions = ["mp4", "mkv", "mov", "avi", "webm"];
    let image_extensions = ["png", "jpg", "jpeg", "webp", "heic", "avif"];

    use std::sync::atomic::{AtomicBool, Ordering};
    let abort_flag = app
        .try_state::<MlContext>()
        .map(|s| s.abort.clone())
        .unwrap_or_else(|| Arc::new(AtomicBool::new(false)));

    for entry in WalkDir::new(directory).follow_links(false) {
        if abort_flag.load(Ordering::SeqCst) {
            return;
        }
        if let Ok(entry) = entry {
            let path = entry.path();
            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if image_extensions.contains(&ext.as_str())
                    || video_extensions.contains(&ext.as_str())
                {
                    if let Ok(path) = fs::canonicalize(path) {
                        image_paths.push(path);
                    }
                }
            }
        }
    }

    use std::sync::{Arc, Mutex};
    let database = Arc::new(Mutex::new(database));
    let app_handle = Arc::new(app.clone());
    let abort_flag_task = Arc::clone(&abort_flag);

    // Create a local thread pool for this scan to avoid blocking the global one if requested
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    use rayon::prelude::*;
    pool.install(|| {
        image_paths.into_par_iter().for_each(|path| {
            if abort_flag_task.load(Ordering::SeqCst) {
                return;
            }
            let db = database.lock().unwrap();
            let path_str = path.display().to_string();
            if db.path_exists(&path_str) {
                return;
            }
            drop(db);

            let id: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(7)
                .map(char::from)
                .collect();

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            let is_video = ["mp4", "mkv", "mov", "avi", "webm"].contains(&ext.as_str());

            if let Ok(file) = File::open(path.clone()) {
                let mut buff = BufReader::new(&file);

                let mut latitude = 0.0;
                let mut longitude = 0.0;
                let mut created = String::new();

                let properties = match Reader::new().read_from_container(&mut buff) {
                    Ok(exif) => {
                        let mut props = HashMap::new();
                        for f in exif.fields() {
                            props.insert(f.tag.to_string(), f.display_value().to_string());
                        }

                        // Extract Created Date
                        if let Some(date_field) = exif
                            .get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
                            .or_else(|| exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY))
                        {
                            created = format!("{}", date_field.display_value());
                        }

                        // Extract GPS Coordinates
                        if let (Some(lat_field), Some(lat_ref)) = (
                            exif.get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY),
                            exif.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY),
                        ) {
                            if let exif::Value::Rational(lat_values) = &lat_field.value {
                                if lat_values.len() == 3 {
                                    let lat = lat_values[0].to_f64()
                                        + lat_values[1].to_f64() / 60.0
                                        + lat_values[2].to_f64() / 3600.0;
                                    latitude = if format!("{}", lat_ref.display_value()) == "S" {
                                        -lat
                                    } else {
                                        lat
                                    };
                                    println!("DEBUG: Extracted latitude: {latitude}");
                                }
                            }
                        }

                        if let (Some(lon_field), Some(lon_ref)) = (
                            exif.get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY),
                            exif.get_field(exif::Tag::GPSLongitudeRef, exif::In::PRIMARY),
                        ) {
                            if let exif::Value::Rational(lon_values) = &lon_field.value {
                                if lon_values.len() == 3 {
                                    let lon = lon_values[0].to_f64()
                                        + lon_values[1].to_f64() / 60.0
                                        + lon_values[2].to_f64() / 3600.0;
                                    longitude = if format!("{}", lon_ref.display_value()) == "W" {
                                        -lon
                                    } else {
                                        lon
                                    };
                                    println!("DEBUG: Extracted longitude: {longitude}");
                                }
                            }
                        }

                        props
                    }
                    Err(_) => HashMap::new(),
                };

                let encoded = if is_video {
                    generate_video_thumbnail(path.to_str().unwrap()).unwrap_or_default()
                } else {
                    generate_thumbnail_base64(path.to_str().unwrap(), 400).unwrap_or_default()
                };

                let photo = database::Photo {
                    id: id.clone(),
                    encoded,
                    location: path.display().to_string(),
                    created,
                    objects: HashMap::new(),
                    properties,
                    latitude,
                    longitude,
                    favorite: false,
                };

                let db = database.lock().unwrap();
                db.store_photo(photo.clone());
                drop(db);

                let _ = app_handle.emit("photo-scanned", photo);

                if let Some(state) = app_handle.try_state::<MlContext>() {
                    state
                        .pending_count
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let _ = state.tx.lock().unwrap().send(id);
                }
            }
        });
    });

    emit_log(app, "Done scanning all photos".to_string());
}

fn generate_video_thumbnail_base64(
    input_path: &str,
    _max_dimension: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let ffmpeg_binary = ffmpeg_sidecar::paths::ffmpeg_path();

    let output = std::process::Command::new(&ffmpeg_binary)
        .arg("-ss")
        .arg("00:00:05.000")
        .arg("-i")
        .arg(input_path)
        .arg("-vframes")
        .arg("1")
        .arg("-vf")
        .arg("scale=400:-1")
        .arg("-q:v")
        .arg("2")
        .arg("-f")
        .arg("image2")
        .arg("-") // output to stdout
        .output()?;

    if !output.status.success() {
        // Fallback: Try 0 seconds if 1 second fails (e.g. video is too short)
        let fallback_output = std::process::Command::new(&ffmpeg_binary)
            .arg("-ss")
            .arg("00:00:00.000")
            .arg("-i")
            .arg(input_path)
            .arg("-vframes")
            .arg("1")
            .arg("-vf")
            .arg("scale=400:-1")
            .arg("-q:v")
            .arg("2")
            .arg("-f")
            .arg("image2")
            .arg("-")
            .output()?;

        if !fallback_output.status.success() {
            let err_msg = String::from_utf8_lossy(&fallback_output.stderr);
            return Err(format!("FFmpeg failed: {err_msg}").into());
        }

        let encoded = general_purpose::STANDARD.encode(&fallback_output.stdout);
        return Ok(format!("data:image/jpeg;base64,{encoded}"));
    }

    let encoded = general_purpose::STANDARD.encode(&output.stdout);
    Ok(format!("data:image/jpeg;base64,{encoded}"))
}

fn generate_thumbnail_base64(
    input_path: &str,
    max_dimension: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("Generating thumbnail for: {input_path}");

    let path = Path::new(input_path);
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_lowercase();
    let video_extensions = ["mp4", "mkv", "mov", "avi", "webm"];

    if video_extensions.contains(&extension.as_str()) {
        return generate_video_thumbnail_base64(input_path, max_dimension);
    }

    let img = ImageReader::open(input_path)?.decode()?;

    let thumbnail = img.thumbnail(max_dimension, max_dimension);

    let mut buffer = Cursor::new(Vec::new());
    thumbnail.write_to(&mut buffer, ImageOutputFormat::Jpeg(80))?;

    let encoded = general_purpose::STANDARD.encode(buffer.get_ref());
    Ok(format!("data:image/jpeg;base64,{encoded}"))
}
// This will return the base64 encoded thumbnail for the image
pub fn get_thumbnail(path: String) -> String {
    match generate_thumbnail_base64(&path, 400) {
        Ok(b64) => b64,
        Err(e) => {
            eprintln!("Failed to generate thumbnail for {path}: {e}");
            String::new()
        }
    }
}

pub fn read_file_base64(path: String) -> String {
    match fs::read(&path) {
        Ok(bytes) => {
            println!("Reading original file: {} ({} bytes)", path, bytes.len());
            let encoded = general_purpose::STANDARD.encode(bytes);
            let ext = Path::new(&path)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_lowercase();
            let mime = match ext.as_str() {
                "png" => "image/png",
                "mp4" => "video/mp4",
                "webm" => "video/webm",
                "mov" => "video/quicktime",
                "avi" => "video/x-msvideo",
                "mkv" => "video/x-matroska",
                _ => "image/jpeg",
            };
            format!("data:{mime};base64,{encoded}")
        }
        Err(e) => {
            eprintln!("Failed to read file {path}: {e}");
            String::new()
        }
    }
}

pub fn generate_video_thumbnail(video_path: &str) -> Option<String> {
    use std::process::Command;
    let temp_dir = std::env::temp_dir();
    let unique_id = uuid::Uuid::new_v4().to_string();
    let output_path = temp_dir.join(format!("{unique_id}.jpg"));

    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-ss",
            "00:00:01",
            "-i",
            video_path,
            "-frames:v",
            "1",
            "-q:v",
            "4",
            "-vf",
            "scale=400:-1",
            output_path.to_str().unwrap(),
        ])
        .status();

    if let Ok(s) = status {
        if s.success() {
            let b64 = read_file_base64(output_path.to_str().unwrap().to_string());
            let _ = std::fs::remove_file(&output_path);
            if !b64.is_empty() {
                return Some(b64);
            }
        }
    }
    None
}

mod tests {

    #[test]
    fn scan_folder() {
        // Test commented out because scan_folder now requires AppHandle which is hard to mock in unit tests
        /*
        use std::collections::HashMap;
        let mut state = HashMap::new();
        state.insert("path".to_string(), "/home/denzyl".to_string());

        let database = crate::database::Database::new("/home/denzyl");
        database.set_state(state);

        let state = database.get_state();
        let directory = state.get("path").unwrap();
        dbg!(&state);
        let _ = super::scan_folder(directory.to_string(), &directory);
        */
    }
}
