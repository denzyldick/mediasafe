use crate::database;
use exif::Reader;
use fast_image_resize::images::Image;
use fast_image_resize::ResizeOptions;
use fast_image_resize::Resizer;
use image_compressor::compressor::Compressor;
use image_compressor::Factor;
use jwalk::WalkDir;
use rand::{distributions::Alphanumeric, Rng};
use rustc_serialize::base64::{ToBase64, MIME};

use std::collections::HashMap;
use std::fs;
use std::fs::File;

use std::io::BufReader;
use std::io::Read;
use std::string::String;

pub fn scan_folder(directory: String, path: &str) {
    let database = database::Database::new(path);
    println!("Scanning all files in: {}", directory);
    for entry in WalkDir::new(directory).follow_links(false) {
        let entry = entry.unwrap();

        let path = entry.path();
        let metadata = fs::metadata(&path).unwrap();
        let file_name = match path.file_name() {
            Some(f) => String::from(f.to_str().unwrap()),
            None => String::from(""),
        };
        if (file_name != "." || !file_name.is_empty()) && metadata.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "png" || extension == "jpg" {
                    let path = fs::canonicalize(path).unwrap();

                    match fs::canonicalize(path) {
                        Ok(path) => {
                            let id: String = rand::thread_rng()
                                .sample_iter(&Alphanumeric)
                                .take(7)
                                .map(char::from)
                                .collect();

                            let file = File::open(path.clone()).unwrap();
                            let mut buff = BufReader::new(&file);
                            let propeties = match Reader::new().read_from_container(&mut buff) {
                                Ok(exif) => {
                                    let mut props = HashMap::new();
                                    for f in exif.fields() {
                                        println!("{}", f.tag.to_string());
                                        props.insert(
                                            f.tag.to_string(),
                                            f.display_value().to_string(),
                                        );
                                    }
                                    props
                                }
                                Err(_err) => HashMap::new(),
                            };

                            let photo = database::Photo {
                                id, // This must be the hash of the file instead of the
                                encoded: get_thumbnail(path.display().to_string()),
                                location: path.display().to_string(),
                                objects: HashMap::new(),
                                properties: propeties,
                            };
                            database.store_photo(photo)
                        }
                        Err(_err) => {}
                    }
                }
            }
        }
    }
    println!("Done scanning all photos");
}

pub fn get_thumbnail(path: String) -> String {
    let mut file = File::open(path.clone()).unwrap();

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    buffer.to_base64(MIME)
}

mod tests {

    #[test]
    fn scan_folder() {
        crate::file::scan_folder(String::from("/home/denzyl"), "/home/denzyl/");
    }
}
