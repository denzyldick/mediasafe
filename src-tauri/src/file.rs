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

///
/// This is will scan a folder recursively and store all the images in the database.
pub fn scan_folder(directory: String, path: &str) {
    let database = database::Database::new(path);
    println!("Scanning all files in: {}", directory);
    for entry in WalkDir::new(directory).follow_links(false) {
        let entry = entry.unwrap();

        let path = entry.path();
        println!("Checking file: {:?}", path);
        let metadata = fs::metadata(&path).unwrap();
        let file_name = match path.file_name() {
            Some(f) => String::from(f.to_str().unwrap()),
            None => String::from(""),
        };
        if (file_name != "." || !file_name.is_empty()) && metadata.is_file() {
            if let Some(extension) = path.extension() {
                println!("Extension: {:?}", extension);
                if extension == "png"
                    || extension == "jpg"
                    || extension == "jpeg"
                    || extension == "JPG"
                    || extension == "PNG"
                {
                    match fs::canonicalize(path) {
                        Ok(path) => {
                            let id: String = rand::thread_rng()
                                .sample_iter(&Alphanumeric)
                                .take(7)
                                .map(char::from)
                                .collect();

                            if let Ok(file) = File::open(path.clone()) {
                                let mut buff = BufReader::new(&file);
                                let propeties = match Reader::new().read_from_container(&mut buff) {
                                    Ok(exif) => {
                                        let mut props = HashMap::new();
                                        for f in exif.fields() {
                                            println!("{}", f.tag);
                                            props.insert(
                                                f.tag.to_string(),
                                                f.display_value().to_string(),
                                            );
                                        }
                                        dbg!(&props);
                                        props
                                    }
                                    Err(_err) => HashMap::new(),
                                };

                                let encoded =
                                    match generate_thumbnail_base64(path.to_str().unwrap(), 400) {
                                        Ok(b64) => b64,
                                        Err(e) => {
                                            println!(
                                                "Failed to generate thumbnail for {}: {}",
                                                path.display(),
                                                e
                                            );
                                            // Fallback to empty string or maybe the path if critical,
                                            // but user requested base64. Let's return empty string for encoded
                                            // so it doesn't try to load invalid base64.
                                            // Actually, if we return empty, no image shows.
                                            // Let's fallback to path to be safe?
                                            // User said "You shouldn't show use the path".
                                            // So let's store empty string if it fails.
                                            String::new()
                                        }
                                    };

                                let photo = database::Photo {
                                    id,
                                    encoded,
                                    location: path.display().to_string(),
                                    objects: HashMap::new(),
                                    properties: propeties,
                                    latitude: 0.0,
                                    longitude: 0.0,
                                    favorite: false,
                                };
                                database.store_photo(photo)
                            }
                        }
                        Err(_err) => {}
                    }
                }
            }
        }
    }
    println!("Done scanning all photos");
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
// This will return the file path for the image
pub fn get_thumbnail(path: String) -> String {
    // For now, just return the original path
    // In the future, we could generate and cache actual thumbnails
    path
}

mod tests {

    #[test]
    fn scan_folder() {
        use std::collections::HashMap;
        let mut state = HashMap::new();
        state.insert("path".to_string(), "/home/denzyl".to_string());

        let database = crate::database::Database::new("/home/denzyl");
        database.set_state(state);

        let state = database.get_state();
        let directory = state.get("path").unwrap();
        dbg!(&state);
        let _ = super::scan_folder(directory.to_string(), &directory);
    }
}
