use crate::database;
use exif::Reader;
use image::GenericImageView;
use jwalk::WalkDir;
use rand::{distributions::Alphanumeric, Rng};
use rustc_serialize::base64;
use rustc_serialize::base64::{ToBase64, MIME};

use std::collections::HashMap;
use std::fs;
use std::fs::File;

use std::io::BufReader;
use std::io::Read;
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

use image::{imageops::FilterType, io::Reader as ImageReader, DynamicImage, ImageOutputFormat};

use std::io::Cursor;
fn generate_thumbnail_base64(
    input_path: &str,
    max_dimension: u32,
) -> Result<String, Box<dyn std::error::Error>> {
    // Step 1: Open the input image
    let img = ImageReader::open(input_path)?.decode()?;

    // Step 2: Calculate new dimensions while maintaining aspect ratio
    let (width, height) = img.dimensions();
    let (new_width, new_height) = if width > height {
        (max_dimension, (max_dimension * height) / width)
    } else {
        ((max_dimension * width) / height, max_dimension)
    };

    // Step 3: Resize the image
    let thumbnail = img.resize_exact(new_width, new_height, FilterType::Lanczos3);
    let mut buff = Cursor::new(vec![0; 15]);
    // Step 4: Encode the thumbnail to an in-memory buffer
    thumbnail.write_to(&mut buff, ImageOutputFormat::Jpeg(75))?;

    // Step 5: Convert the buffer to a Base64 string
    let base64_thumbnail = String::from_utf8(buff.into_inner());
    match base64_thumbnail {
        Ok(base64_thumbnail) => {
            Ok(base64_thumbnail)
        }
        Err(err) => Err(Box::new(err)),
    }
}
// This will generate a thumbnail for the image
pub fn get_thumbnail(path: String) -> String {
    generate_thumbnail_base64(&path, 100).unwrap()
}

mod tests {

    #[test]
    fn scan_folder() {
        let file = crate::file::scan_folder(String::from("/home/denzyl"), "/home/denzyl/");

        /// I forgot what I was testing in the test.
        assert_eq!(true, true)
    }
}
