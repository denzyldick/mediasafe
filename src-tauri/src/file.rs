use crate::database;
use exif::Reader;
use image::GenericImageView;
use jwalk::WalkDir;
use rand::{distributions::Alphanumeric, Rng};
use rustc_serialize::base64;
use rustc_serialize::base64::{ToBase64, MIME};
use thumbnailer::ThumbnailSize;

use std::collections::HashMap;
use std::fs;
use std::fs::File;

use std::io::Read;
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

use thumbnailer::create_thumbnails;
extern crate mime;
fn generate_thumbnail_base64(
    input_path: &str,
    max_dimension: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    let file = File::open(input_path).unwrap();
    let reader = BufReader::new(file);
    let mut thumbnails = create_thumbnails(
        reader,
        mime::IMAGE_PNG,
        [ThumbnailSize::Small, ThumbnailSize::Medium],
    )
    .unwrap();

    let thumbnail = thumbnails.pop().unwrap();
    let mut buf = Cursor::new(Vec::new());

    let s = String::from_utf8_lossy(&mut buf.into_inner()).to_string();
    println!("{}r#", s);
    Ok(s)
}
// This will generate a thumbnail for the image
pub fn get_thumbnail(path: String) -> String {
    let base64 = generate_thumbnail_base64(&path, 100);
    match base64 {
        Ok(base64) => base64,
        Err(_err) => String::from("Something# went wrong"),
    }
}

mod tests {

    #[test]
    fn scan_folder() {
        let file = crate::file::scan_folder(String::from("/home/denzyl"), "/home/denzyl/");

        /// I forgot what I was testing in the test.
        assert_eq!(true, true)
    }
}
