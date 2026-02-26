use std::{
    collections::HashMap,
    fs::{self, File},
};

use rusqlite::Connection;
use serde::Serialize;

pub struct Database {
    pub connection: Connection,
}

impl Database {
    pub fn new(config_path: &str) -> Self {
        let path = format!("{}/database.sql", config_path);
        println!("Database.sql location: {}", path);

        // Ensure the directory exists
        let _ = fs::create_dir_all(config_path);

        let file = fs::metadata(&path);
        if let Err(_kind) = file {
            println!("Database located at {} doesns't exists.", path);
            let _ = File::create(&path);
        }
        let conn = Connection::open(format!("{}/database.sql", config_path)).unwrap();

        // Check if photo table needs migration by checking if latitude exists
        if conn.prepare("SELECT latitude FROM photo LIMIT 1").is_err() {
            println!("Detected old schema. Dropping photo table to migrate...");
            let _ = conn.execute("DROP TABLE IF EXISTS photo", ());
        }

        conn.execute(
            "CREATE TABLE IF NOT EXISTS photo (
            id    STRING PRIMARY KEY,
            location  STRING,
            encoded STRING,
            created DATE_TIME,
            latitude REAL,
            longitude REAL
             );
        ",
            (),
        )
        .unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS directory (name STRING);", ())
            .unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS object(
            photo_id STRING,
            class STRING,
            probability STRING
         );",
            (),
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS properties (
                photo_id STRING,
                key STRING,
                value STRING
            );",
            (),
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS device(
                ip STRING,
                name STRING,
                offer STRING
                );",
            (),
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS faces (
                photo_id STRING,
                face_id STRING PRIMARY KEY,
                crop_path STRING
            );",
            (),
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS config(
                key STRING,
                value STRING
                );",
            (),
        )
        .unwrap();
        Self { connection: conn }
    }

    pub fn get_state(&self) -> HashMap<String, String> {
        let mut result = self
            .connection
            .prepare("SELECT key, value FROM config")
            .unwrap();
        result
            .query_map([], |row| {
                let key: String = row.get(0).unwrap();
                // Try to get as string first, if that fails try as i64 and convert
                let value: String = row
                    .get::<_, String>(1)
                    .or_else(|_| row.get::<_, i64>(1).map(|v| v.to_string()))
                    .unwrap();
                Ok((key, value))
            })
            .unwrap()
            .map(|x| x.unwrap())
            .collect()
    }

    pub fn set_state(&self, state: HashMap<String, String>) {
        for (key, value) in state {
            self.connection
                .execute(
                    "INSERT INTO config (key, value) VALUES(?1, ?2)",
                    (&key, &value),
                )
                .unwrap();
        }
    }

    pub fn get_last_scan_time(&self) -> Option<String> {
        let state = self.get_state();
        state.get("last_scan_time").cloned()
    }

    pub fn set_last_scan_time(&self, timestamp: String) {
        self.connection
            .execute(
                "INSERT OR REPLACE INTO config (key, value) VALUES('last_scan_time', ?1)",
                [&timestamp],
            )
            .unwrap();
    }

    pub fn store_photo(&self, photo: Photo) {
        let result = self.connection.execute(
            "INSERT INTO photo(id, location, encoded, latitude, longitude) VALUES(?1, ?2, ?3, ?4, ?5 )",
            (&photo.id, &photo.location, &photo.encoded, &photo.latitude, &photo.longitude),
        );

        if let Ok(_result) = result {
            println!("Photo has been saved");
        }

        if let Err(error) = result {
            println!("{}", error);
        }
        for (object, probablity) in photo.objects {
            self.connection
                .execute(
                    "INSERT INTO object (photo_id, class, probability) VALUES(?1, ?2, ?3)",
                    (&photo.id, &object, &probablity),
                )
                .unwrap();

            println!("Inserting probability");
        }
        println!("Photo stored successfully: {}", photo.id);

        for (key, value) in photo.properties {
            let params = (&photo.id, &key, &value);
            self.connection.execute(
                "INSERT into properties (photo_id, key, value) VALUES(?1, ?2, ?3)",
                params,
            );
        }
    }

    pub fn list_objects(self, query: &str) -> Vec<String> {
        println!("Searching for {}", query);
        let mut objects = Vec::new();

        let mut statement = self
            .connection
            .prepare("SELECT class FROM object WHERE class LIKE :query GROUP BY class ")
            .unwrap();

        let object_iter = statement
            .query_map(&[(":query", &format!("%{query}%"))], |row| {
                let s: String = row.get(0).unwrap();
                Ok(s)
            })
            .unwrap();
        for i in object_iter {
            objects.push(i.unwrap());
        }
        println!("{objects:#?}");
        objects
    }
    pub fn get_photo(self, _id: &str) {
        let sql = "SELECT id,encoded FROM photo WHERE photo.id = :id";
        let mut stmt = self.connection.prepare(sql).unwrap();

        let _ = stmt
            .query_map(&[(":id", &"one")], |_row| Ok(String::from("Hfoaufaea")))
            .unwrap();
    }
    pub fn list_photos(
        self,
        query: &str,
        offset: usize,
        limit: usize,
        favorites_only: bool,
    ) -> Vec<Photo> {
        let mut photos = Vec::new();

        let should_filter_fav = if favorites_only {
            "AND is_fav IS NOT NULL"
        } else {
            ""
        };
        let query_filter = if !query.is_empty() {
            "AND (o.class LIKE ?3 OR p.location LIKE ?3 OR p.id LIKE ?3)"
        } else {
            ""
        };

        // Base query with joins to get favorite status efficiently
        // We use a subquery or join for properties where key='favorite'
        let sql = format!(
            "SELECT 
                p.id, 
                p.location, 
                p.encoded, 
                p.latitude, 
                p.longitude,
                prop.value as is_fav,
                GROUP_CONCAT(o.class || ':' || o.probability) as tags
             FROM photo p 
             LEFT JOIN properties prop ON p.id = prop.photo_id AND prop.key = 'favorite'
             LEFT JOIN object o ON p.id = o.photo_id 
             WHERE 1=1 
             {} 
             {}
             GROUP BY p.id
             ORDER BY p.created DESC, MAX(o.probability) DESC 
             LIMIT ?1, ?2",
            should_filter_fav, query_filter
        );

        let param_offset = offset.to_string();
        let param_limit = limit.to_string();
        let param_query = format!("%{query}%");

        let mut stmt = self.connection.prepare(&sql).unwrap();

        let params: Vec<&dyn rusqlite::ToSql> = if !query.is_empty() {
            vec![&param_offset, &param_limit, &param_query]
        } else {
            vec![&param_offset, &param_limit]
        };

        let photo_iter = stmt
            .query_map(params.as_slice(), |row| {
                let fav_val: Option<String> = row.get(5).ok();
                let is_fav = fav_val.map(|v| v == "true").unwrap_or(false);

                let mut objects = HashMap::new();
                if let Ok(Some(tags_str)) = row.get::<_, Option<String>>(6) {
                    for tag_pair in tags_str.split(',') {
                        let parts: Vec<&str> = tag_pair.split(':').collect();
                        if parts.len() == 2 {
                            let class = parts[0].to_string();
                            let prob = parts[1].parse::<f64>().unwrap_or(1.0);
                            objects.insert(class, prob);
                        }
                    }
                }

                Ok(Photo {
                    id: row.get(0)?,
                    location: row.get(1)?,
                    encoded: row.get(2)?,
                    objects,
                    properties: HashMap::new(),
                    latitude: row.get(3).unwrap_or(0.0),
                    longitude: row.get(4).unwrap_or(0.0),
                    favorite: is_fav,
                })
            })
            .unwrap();

        for p in photo_iter {
            if let Ok(photo) = p {
                photos.push(photo);
            }
        }

        println!("Photos found, {}, {} {}", photos.len(), offset, limit);
        photos
    }

    pub fn toggle_favorite(&self, photo_id: &str) -> bool {
        // Check if currently favorite
        let current_sql = "SELECT value FROM properties WHERE photo_id = ?1 AND key = 'favorite'";
        let mut stmt = self.connection.prepare(current_sql).unwrap();
        let exists = stmt.exists([photo_id]).unwrap_or(false);

        if exists {
            // Remove it
            let _ = self.connection.execute(
                "DELETE FROM properties WHERE photo_id = ?1 AND key = 'favorite'",
                [photo_id],
            );
            return false;
        } else {
            // Add it
            let _ = self.connection.execute(
                "INSERT INTO properties (photo_id, key, value) VALUES(?1, 'favorite', 'true')",
                [photo_id],
            );
            return true;
        }
    }

    pub fn get_all_photos_with_location(&self) -> Vec<Photo> {
        let sql = "SELECT id, location, encoded, latitude, longitude FROM photo WHERE latitude != 0.0 AND longitude != 0.0";
        let mut stmt = self.connection.prepare(sql).unwrap();
        let photo_iter = stmt
            .query_map([], |row| {
                Ok(Photo {
                    id: row.get(0)?,
                    location: row.get(1)?,
                    encoded: row.get(2)?,
                    objects: HashMap::new(),
                    properties: HashMap::new(),
                    latitude: row.get(3).unwrap_or(0.0),
                    longitude: row.get(4).unwrap_or(0.0),
                    favorite: false,
                })
            })
            .unwrap();

        let mut photos = Vec::new();
        for p in photo_iter {
            if let Ok(photo) = p {
                photos.push(photo);
            }
        }
        photos
    }

    pub fn store_face(&self, face: Face) {
        let result = self.connection.execute(
            "INSERT INTO faces(photo_id, face_id, crop_path) VALUES(?1, ?2, ?3)",
            (&face.photo_id, &face.face_id, &face.crop_path),
        );
        if let Err(error) = result {
            println!("Error storing face: {}", error);
        }
    }

    pub fn get_all_faces(&self) -> Vec<Face> {
        let mut stmt = self
            .connection
            .prepare("SELECT photo_id, face_id, crop_path FROM faces GROUP BY face_id")
            .unwrap();
        let face_iter = stmt
            .query_map([], |row| {
                Ok(Face {
                    photo_id: row.get(0)?,
                    face_id: row.get(1)?,
                    crop_path: row.get(2)?,
                })
            })
            .unwrap();

        let mut faces = Vec::new();
        for f in face_iter {
            if let Ok(face) = f {
                faces.push(face);
            }
        }
        faces
    }

    pub(crate) fn list_directories(&self) -> Vec<String> {
        println!("Listing directories form DB");
        let mut stm = self.connection.prepare("SELECT * FROM directory").unwrap();

        let results: Vec<String> = stm
            .query_map((), |row| {
                let s: String = row.get(0).unwrap();
                Ok(s)
            })
            .unwrap()
            .map(|x| x.unwrap())
            .collect();
        println!("Found {} directories", results.len());
        results
    }

    pub(crate) fn remove_directory(&self, path: String) {
        let mut stm = self
            .connection
            .prepare("DELETE FROM directory WHERE name = ?1")
            .unwrap();
        stm.execute([&path]).unwrap();
    }

    pub(crate) fn add_directory(&self, path: &str) {
        println!("Adding directory to DB: {}", path);
        let mut stm = self
            .connection
            .prepare("INSERT INTO directory (name) VALUES(?1)")
            .unwrap();
        match stm.execute([&path]) {
            Ok(_) => println!("Successfully inserted directory"),
            Err(e) => println!("Failed to insert directory: {}", e),
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Photo {
    pub id: String,
    pub location: String,
    pub encoded: String,
    pub objects: HashMap<String, f64>,
    pub properties: HashMap<String, String>,
    pub latitude: f64,
    pub longitude: f64,
    pub favorite: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Face {
    pub photo_id: String,
    pub face_id: String,
    pub crop_path: String,
}
//use image::io::Reader as ImageReader;
//use image::{DynamicImage, GenericImageView};
//use tch::{nn, vision::imagenet, Device as DeviceTch, Tensor};

impl Photo {
    /*  pub(crate) fn clasify_image(&self) {
            // Step 1: Load the model
           /* let device = Device::cuda_if_available(); // Use GPU if available
            let model = imagenet::resnet18(&nn::VarStore::new(device))?;

            // Step 2: Load and preprocess the image
            let img = ImageReader::open("path/to/image.jpg")?.decode()?.to_rgb8();
            let (width, height) = img.dimensions();
            let resized = image::imageops::resize(&img, 224, 224, image::imageops::FilterType::Nearest);

            let img_tensor = Tensor::of_slice(&resized.to_vec())
                .view((1, 224, 224, 3))
                .permute(&[0, 3, 1, 2])
                .to_device(device)
                .to_kind(tch::Kind::Float) / 255.0;

            // Step 3: Perform classification
            let output = model.forward_t(&img_tensor, false);
            let class_id = output.argmax(1, false).int64_value(&[0]);
            let classes = imagenet::load_classes()?;

            // Step 4: Print the result
            println!("Predicted class: {}", classes.get(class_id as usize).unwrap_or(&"unknown".to_string()));

    */
            Ok(())    }
        */
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_init() {
        let _database = Database::new("./tests");
    }

    #[test]
    fn clasify_image() {
        let _photo = Photo {
            id: String::from("1"),
            location: String::from(""),
            encoded: String::from(""),
            objects: HashMap::new(),
            properties: HashMap::new(),
            latitude: 0.0,
            longitude: 0.0,
            favorite: false,
        };
    }
}
