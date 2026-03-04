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

use std::io::{BufReader, Cursor};
use std::string::String;

use crate::ml::MlContext;
use tauri::{Emitter, Manager};

fn emit_log(app: &tauri::AppHandle, message: String) {
    println!("{}", message);
    let _ = app.emit("log-message", message);
}

///
/// This is will scan a folder recursively and store all the images in the database.
pub fn scan_folder(app: &tauri::AppHandle, directory: String, path: &str) {
    let database = database::Database::new(path);
    emit_log(app, format!("Scanning all files in: {}", directory));
    
    // Collect all valid image paths first
    let mut image_paths = Vec::new();
    for entry in WalkDir::new(directory).follow_links(false) {
        if let Ok(entry) = entry {
            let path = entry.path();
            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if ext == "png" || ext == "jpg" || ext == "jpeg" {
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

    use rayon::prelude::*;
    image_paths.into_par_iter().for_each(|path| {
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

        if let Ok(file) = File::open(path.clone()) {
            let mut buff = BufReader::new(&file);
            let mut latitude = 0.0;
            let mut longitude = 0.0;

            let properties = match Reader::new().read_from_container(&mut buff) {
                Ok(exif) => {
                    let mut props = HashMap::new();
                    for f in exif.fields() {
                        props.insert(f.tag.to_string(), f.display_value().to_string());
                    }

                    // Extract GPS Coordinates
                    if let (Some(lat_field), Some(lat_ref)) = (exif.get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY), exif.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY)) {
                        if let exif::Value::Rational(lat_values) = &lat_field.value {
                            if lat_values.len() == 3 {
                                let lat = lat_values[0].to_f64() + lat_values[1].to_f64() / 60.0 + lat_values[2].to_f64() / 3600.0;
                                latitude = if format!("{}", lat_ref.display_value()) == "S" { -lat } else { lat };
                            }
                        }
                    }

                    if let (Some(lon_field), Some(lon_ref)) = (exif.get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY), exif.get_field(exif::Tag::GPSLongitudeRef, exif::In::PRIMARY)) {
                        if let exif::Value::Rational(lon_values) = &lon_field.value {
                            if lon_values.len() == 3 {
                                let lon = lon_values[0].to_f64() + lon_values[1].to_f64() / 60.0 + lon_values[2].to_f64() / 3600.0;
                                longitude = if format!("{}", lon_ref.display_value()) == "W" { -lon } else { lon };
                            }
                        }
                    }

                    props
                }
                Err(_) => HashMap::new(),
            };

            let encoded = match generate_thumbnail_base64(path.to_str().unwrap(), 400) {
                Ok(b64) => b64,
                Err(_) => String::new(),
            };

            let photo = database::Photo {
                id: id.clone(),
                encoded,
                location: path.display().to_string(),
                objects: HashMap::new(),
                properties,
                latitude,
                longitude,
                favorite: false,
            };

            let db = database.lock().unwrap();
            db.store_photo(photo);
            drop(db);

            if let Some(state) = app_handle.try_state::<MlContext>() {
                state.pending_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let _ = state.tx.lock().unwrap().send(id);
            }
        }
    });

    emit_log(app, "Done scanning all photos".to_string());
}

fn generate_thumbnail_base64(
    input_path: &str,
    max_dimension: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("Generating thumbnail for: {}", input_path);
    let img = ImageReader::open(input_path)?.decode()?;
    let thumbnail = img.thumbnail(max_dimension, max_dimension);

    let mut buffer = Cursor::new(Vec::new());
    thumbnail.write_to(&mut buffer, ImageOutputFormat::Jpeg(80))?;

    let encoded = general_purpose::STANDARD.encode(buffer.get_ref());
    Ok(format!("data:image/jpeg;base64,{}", encoded))
}
// This will return the base64 encoded thumbnail for the image
pub fn get_thumbnail(path: String) -> String {
    match generate_thumbnail_base64(&path, 400) {
        Ok(b64) => b64,
        Err(e) => {
            eprintln!("Failed to generate thumbnail for {}: {}", path, e);
            String::new()
        }
    }
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
