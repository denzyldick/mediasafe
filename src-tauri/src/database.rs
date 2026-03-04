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
            println!("Database located at {} doesn't exist.", path);
            let _ = File::create(&path);
        }
        
        let conn = Connection::open(format!("{}/database.sql", config_path))
            .expect("Failed to open database connection");

        // Check if photo table needs migration by checking if latitude exists
        if conn.prepare("SELECT latitude FROM photo LIMIT 1").is_err() {
            println!("Detected old photo schema. Dropping photo table to migrate...");
            let _ = conn.execute("DROP TABLE IF EXISTS photo", ());
        }

        // Check if faces table needs migration by checking if encoded exists
        if conn.prepare("SELECT encoded FROM faces LIMIT 1").is_err() {
            println!("Detected old faces schema. Dropping faces table to migrate...");
            let _ = conn.execute("DROP TABLE IF EXISTS faces", ());
        }

        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS photo (
            id    STRING PRIMARY KEY,
            location  STRING,
            encoded STRING,
            created DATE_TIME,
            latitude REAL,
            longitude REAL
             );",
            (),
        );
        let _ = conn.execute("CREATE TABLE IF NOT EXISTS directory (name STRING);", ());
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS object(
            photo_id STRING,
            class STRING,
            probability STRING
         );",
            (),
        );

        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS properties (
                photo_id STRING,
                key STRING,
                value STRING
            );",
            (),
        );

        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS device(
                ip STRING,
                name STRING,
                offer STRING
                );",
            (),
        );

        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS faces (
                photo_id STRING,
                face_id STRING PRIMARY KEY,
                crop_path STRING,
                encoded STRING,
                person_id STRING
            );",
            (),
        );

        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS people (
                id STRING PRIMARY KEY,
                name STRING
            );",
            (),
        );

        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS config(
                key STRING,
                value STRING
                );",
            (),
        );

        // Add indices for performance
        let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_object_photo_id ON object(photo_id)", ());
        let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_object_class ON object(class)", ());
        let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_faces_photo_id ON faces(photo_id)", ());
        let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_faces_person_id ON faces(person_id)", ());

        Self { connection: conn }
    }

    pub fn get_state(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Ok(mut stmt) = self.connection.prepare("SELECT key, value FROM config") {
            let rows = stmt.query_map([], |row| {
                let key: String = row.get(0)?;
                let value: String = row.get::<_, String>(1)
                    .or_else(|_| row.get::<_, i64>(1).map(|v| v.to_string()))?;
                Ok((key, value))
            });
            
            if let Ok(rows) = rows {
                for row in rows {
                    if let Ok((key, value)) = row {
                        map.insert(key, value);
                    }
                }
            }
        }
        map
    }

    pub fn set_state(&self, state: HashMap<String, String>) {
        for (key, value) in state {
            let _ = self.connection.execute(
                "INSERT INTO config (key, value) VALUES(?1, ?2)",
                (&key, &value),
            );
        }
    }

    pub fn get_last_scan_time(&self) -> Option<String> {
        let state = self.get_state();
        state.get("last_scan_time").cloned()
    }

    pub fn set_last_scan_time(&self, timestamp: String) {
        let _ = self.connection.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES('last_scan_time', ?1)",
            [&timestamp],
        );
    }

    pub fn store_photo(&self, photo: Photo) {
        let result = self.connection.execute(
            "INSERT INTO photo(id, location, encoded, latitude, longitude, created) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            (&photo.id, &photo.location, &photo.encoded, &photo.latitude, &photo.longitude, &photo.created),
        );

        if result.is_ok() {
            println!("Photo stored: {}", photo.id);
        } else if let Err(e) = result {
            println!("Error storing photo {}: {}", photo.id, e);
        }

        for (object, probability) in photo.objects {
            let _ = self.connection.execute(
                "INSERT INTO object (photo_id, class, probability) VALUES(?1, ?2, ?3)",
                (&photo.id, &object, &probability.to_string()),
            );
        }

        for (key, value) in photo.properties {
            let _ = self.connection.execute(
                "INSERT into properties (photo_id, key, value) VALUES(?1, ?2, ?3)",
                (&photo.id, &key, &value),
            );
        }
    }

    pub fn list_objects(self, query: &str) -> Vec<String> {
        let mut objects = Vec::new();
        if let Ok(mut stmt) = self.connection.prepare("SELECT class FROM object WHERE class LIKE :query GROUP BY class") {
            let iter = stmt.query_map(&[(":query", &format!("%{query}%"))], |row| row.get(0));
            if let Ok(iter) = iter {
                for item in iter {
                    if let Ok(s) = item {
                        objects.push(s);
                    }
                }
            }
        }
        objects
    }

    pub fn list_photos(
        self,
        query: &str,
        offset: usize,
        limit: usize,
        favorites_only: bool,
    ) -> Vec<Photo> {
        let mut photos = Vec::new();

        let should_filter_fav = if favorites_only { "AND is_fav IS NOT NULL" } else { "" };
        let query_filter = if !query.is_empty() { "AND (o.class LIKE ?3 OR p.location LIKE ?3 OR p.id LIKE ?3)" } else { "" };

        let sql = format!(
            "SELECT 
                p.id, p.location, p.encoded, p.latitude, p.longitude,
                prop.value as is_fav,
                GROUP_CONCAT(o.class || ':' || o.probability) as tags,
                p.created
             FROM photo p 
             LEFT JOIN properties prop ON p.id = prop.photo_id AND prop.key = 'favorite'
             LEFT JOIN object o ON p.id = o.photo_id 
             WHERE 1=1 {} {}
             GROUP BY p.id
             ORDER BY p.created DESC, MAX(o.probability) DESC 
             LIMIT ?1, ?2",
            should_filter_fav, query_filter
        );

        let mut stmt = match self.connection.prepare(&sql) {
            Ok(s) => s,
            Err(_) => return photos,
        };

        let param_offset = offset.to_string();
        let param_limit = limit.to_string();
        let param_query = format!("%{query}%");

        let params: Vec<&dyn rusqlite::ToSql> = if !query.is_empty() {
            vec![&param_offset, &param_limit, &param_query]
        } else {
            vec![&param_offset, &param_limit]
        };

        let photo_iter = stmt.query_map(params.as_slice(), |row| {
            self.row_to_photo(row)
        });

        if let Ok(iter) = photo_iter {
            for p in iter {
                if let Ok(photo) = p {
                    photos.push(photo);
                }
            }
        }
        photos
    }

    fn row_to_photo(&self, row: &rusqlite::Row) -> rusqlite::Result<Photo> {
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
            created: row.get(7).unwrap_or_default(),
            objects,
            properties: HashMap::new(),
            latitude: row.get(3).unwrap_or(0.0),
            longitude: row.get(4).unwrap_or(0.0),
            favorite: row.get::<_, Option<String>>(5).ok().flatten().as_deref() == Some("true"),
        })
    }

    pub fn toggle_favorite(&self, photo_id: &str) -> bool {
        let current_sql = "SELECT 1 FROM properties WHERE photo_id = ?1 AND key = 'favorite'";
        let exists = self.connection.prepare(current_sql)
            .and_then(|mut s| s.exists([photo_id]))
            .unwrap_or(false);

        if exists {
            let _ = self.connection.execute(
                "DELETE FROM properties WHERE photo_id = ?1 AND key = 'favorite'",
                [photo_id],
            );
            false
        } else {
            let _ = self.connection.execute(
                "INSERT INTO properties (photo_id, key, value) VALUES(?1, 'favorite', 'true')",
                [photo_id],
            );
            true
        }
    }

    pub fn get_all_photos_with_location(&self) -> Vec<Photo> {
        let mut photos = Vec::new();
        let sql = "SELECT id, location, encoded, latitude, longitude, created FROM photo WHERE latitude != 0.0 AND longitude != 0.0";
        if let Ok(mut stmt) = self.connection.prepare(sql) {
            let iter = stmt.query_map([], |row| {
                Ok(Photo {
                    id: row.get(0)?,
                    location: row.get(1)?,
                    encoded: row.get(2)?,
                    created: row.get(5).unwrap_or_default(),
                    objects: HashMap::new(),
                    properties: HashMap::new(),
                    latitude: row.get(3).unwrap_or(0.0),
                    longitude: row.get(4).unwrap_or(0.0),
                    favorite: false,
                })
            });
            if let Ok(iter) = iter {
                for p in iter {
                    if let Ok(photo) = p {
                        photos.push(photo);
                    }
                }
            }
        }
        photos
    }

    pub fn store_face(&self, face: Face) {
        let _ = self.connection.execute(
            "INSERT INTO faces(photo_id, face_id, crop_path, encoded) VALUES(?1, ?2, ?3, ?4)",
            (&face.photo_id, &face.face_id, &face.crop_path, &face.encoded),
        );
    }

    pub fn get_all_faces(&self) -> Vec<Face> {
        let mut faces = Vec::new();
        if let Ok(mut stmt) = self.connection.prepare("SELECT photo_id, face_id, crop_path, person_id, encoded FROM faces GROUP BY face_id") {
            let iter = stmt.query_map([], |row| {
                Ok(Face {
                    photo_id: row.get(0)?,
                    face_id: row.get(1)?,
                    crop_path: row.get(2)?,
                    person_id: row.get(3).ok(),
                    encoded: row.get(4).unwrap_or_default(),
                })
            });
            if let Ok(iter) = iter {
                for f in iter {
                    if let Ok(face) = f {
                        faces.push(face);
                    }
                }
            }
        }
        faces
    }

    pub fn get_people(&self) -> Vec<PersonWithFace> {
        let mut people = Vec::new();
        let sql = "
            SELECT p.id, p.name, f.crop_path, f.face_id, f.encoded
            FROM people p
            LEFT JOIN faces f ON p.id = f.person_id
            GROUP BY p.id
        ";
        if let Ok(mut stmt) = self.connection.prepare(sql) {
            let iter = stmt.query_map([], |row| {
                Ok(PersonWithFace {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    representative_crop: row.get(2).ok(),
                    representative_face_id: row.get(3).ok(),
                    encoded: row.get(4).ok(),
                })
            });
            if let Ok(iter) = iter {
                for p in iter {
                    if let Ok(person) = p {
                        people.push(person);
                    }
                }
            }
        }
        people
    }

    pub fn assign_name_to_face(&self, face_id: &str, name: &str) -> String {
        // Check if person already exists by name
        let mut stmt = self.connection.prepare("SELECT id FROM people WHERE name = ?1").unwrap();
        let person_id: Option<String> = stmt.query_row([name], |row| row.get(0)).ok();

        let id = match person_id {
            Some(existing_id) => existing_id,
            None => {
                let new_id = uuid::Uuid::new_v4().to_string();
                let _ = self.connection.execute("INSERT INTO people (id, name) VALUES (?1, ?2)", (&new_id, name));
                new_id
            }
        };

        let _ = self.connection.execute("UPDATE faces SET person_id = ?1 WHERE face_id = ?2", (&id, face_id));
        id
    }

    pub fn get_unnamed_faces(&self) -> Vec<Face> {
        let mut faces = Vec::new();
        if let Ok(mut stmt) = self.connection.prepare("SELECT photo_id, face_id, crop_path, person_id, encoded FROM faces WHERE person_id IS NULL") {
            let iter = stmt.query_map([], |row| {
                Ok(Face {
                    photo_id: row.get(0)?,
                    face_id: row.get(1)?,
                    crop_path: row.get(2)?,
                    person_id: None,
                    encoded: row.get(4).unwrap_or_default(),
                })
            });
            if let Ok(iter) = iter {
                for f in iter {
                    if let Ok(face) = f {
                        faces.push(face);
                    }
                }
            }
        }
        faces
    }

    pub fn get_photos_for_person(&self, person_id: &str) -> Vec<Photo> {
        let mut photos = Vec::new();
        let sql = "
            SELECT p.id, p.location, p.encoded, p.latitude, p.longitude, p.created
            FROM photo p
            JOIN faces f ON p.id = f.photo_id
            WHERE f.person_id = ?1
            GROUP BY p.id
        ";
        if let Ok(mut stmt) = self.connection.prepare(sql) {
            let iter = stmt.query_map([person_id], |row| {
                Ok(Photo {
                    id: row.get(0)?,
                    location: row.get(1)?,
                    encoded: row.get(2)?,
                    created: row.get(5).unwrap_or_default(),
                    objects: HashMap::new(),
                    properties: HashMap::new(),
                    latitude: row.get(3).unwrap_or(0.0),
                    longitude: row.get(4).unwrap_or(0.0),
                    favorite: false, // Simplified for this view
                })
            });
            if let Ok(iter) = iter {
                for p in iter {
                    if let Ok(photo) = p {
                        photos.push(photo);
                    }
                }
            }
        }
        photos
    }

    pub(crate) fn list_directories(&self) -> Vec<String> {
        let mut results = Vec::new();
        if let Ok(mut stm) = self.connection.prepare("SELECT name FROM directory") {
            let iter = stm.query_map((), |row| row.get(0));
            if let Ok(iter) = iter {
                for s in iter {
                    if let Ok(val) = s {
                        results.push(val);
                    }
                }
            }
        }
        results
    }

    pub(crate) fn remove_directory(&self, path: String) {
        let _ = self.connection.execute("DELETE FROM directory WHERE name = ?1", [&path]);
    }

    pub(crate) fn add_directory(&self, path: &str) {
        let _ = self.connection.execute("INSERT INTO directory (name) VALUES(?1)", [&path]);
    }

    pub fn path_exists(&self, path: &str) -> bool {
        self.connection.prepare("SELECT 1 FROM photo WHERE location = ?1")
            .and_then(|mut s| s.exists([path]))
            .unwrap_or(false)
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Photo {
    pub id: String,
    pub location: String,
    pub encoded: String,
    pub created: String,
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
    pub encoded: String,
    pub person_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PersonWithFace {
    pub id: String,
    pub name: String,
    pub representative_crop: Option<String>,
    pub representative_face_id: Option<String>,
    pub encoded: Option<String>,
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
