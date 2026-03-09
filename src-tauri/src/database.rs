use std::{
    collections::HashMap,
    fs::{self},
};

use rusqlite::Connection;
use serde::Serialize;

pub struct Database {
    pub connection: Connection,
}
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct PhotoSyncInfo {
    pub id: String,
    pub location: String,
    pub created: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FaceWithPerson {
    pub photo_id: String,
    pub face_id: String,
    pub crop_path: String,
    pub encoded: String,
    pub person_id: Option<String>,
    pub person_name: Option<String>,
}

impl Database {
    pub fn get_photo_sync_info(&self) -> Vec<PhotoSyncInfo> {
        let mut results = Vec::new();
        let sql = "SELECT id, location, created FROM photo";
        if let Ok(mut stmt) = self.connection.prepare(sql) {
            let iter = stmt.query_map([], |row| {
                Ok(PhotoSyncInfo {
                    id: row.get(0)?,
                    location: row.get(1)?,
                    created: row.get(2).unwrap_or_default(),
                })
            });
            if let Ok(iter) = iter {
                for p in iter.flatten() {
                    results.push(p);
                }
            }
        }
        results
    }

    pub fn new(config_path: &str) -> Self {
        let path = format!("{config_path}/siegu.db");
        let _ = fs::create_dir_all(config_path);
        let conn = Connection::open(&path).expect("Failed to open database connection");

        let _ = conn.execute("CREATE TABLE IF NOT EXISTS photo (id STRING PRIMARY KEY, location STRING, encoded STRING, created DATE_TIME, latitude REAL, longitude REAL);", ());

        // Simple migration: try to add columns if they don't exist (ignore errors if they do)
        let _ = conn.execute("ALTER TABLE photo ADD COLUMN latitude REAL;", ());
        let _ = conn.execute("ALTER TABLE photo ADD COLUMN longitude REAL;", ());
        let _ = conn.execute("ALTER TABLE photo ADD COLUMN created DATE_TIME;", ());

        let _ = conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_photo_location ON photo(location);",
            (),
        );
        let _ = conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_photo_created ON photo(created);",
            (),
        );
        let _ = conn.execute("CREATE TABLE IF NOT EXISTS directory (name STRING);", ());
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS object(photo_id STRING, class STRING, probability STRING);",
            (),
        );
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS properties (photo_id STRING, key STRING, value STRING);",
            (),
        );
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS device(ip STRING, name STRING, offer STRING);",
            (),
        );
        let _ = conn.execute("CREATE TABLE IF NOT EXISTS faces (photo_id STRING, face_id STRING PRIMARY KEY, crop_path STRING, encoded STRING, embedding BLOB, person_id STRING);", ());
        let _ = conn.execute("CREATE TABLE IF NOT EXISTS people (id STRING PRIMARY KEY, name STRING, embedding BLOB);", ());
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS config(key STRING, value STRING);",
            (),
        );
        let _ = conn.execute("CREATE TABLE IF NOT EXISTS logs (timestamp DATETIME DEFAULT CURRENT_TIMESTAMP, level STRING, message TEXT);", ());

        Self { connection: conn }
    }

    pub fn get_state(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Ok(mut stmt) = self.connection.prepare("SELECT key, value FROM config") {
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            }) {
                for row in rows.flatten() {
                    map.insert(row.0, row.1);
                }
            }
        }
        map
    }

    pub fn store_log(&self, level: &str, message: &str) {
        let _ = self.connection.execute(
            "INSERT INTO logs (level, message) VALUES (?1, ?2)",
            (level, message),
        );
    }

    pub fn get_logs(&self, limit: usize) -> Vec<LogEntry> {
        let mut logs = Vec::new();
        let sql = "SELECT timestamp, level, message FROM logs ORDER BY timestamp DESC LIMIT ?1";
        if let Ok(mut stmt) = self.connection.prepare(sql) {
            if let Ok(iter) = stmt.query_map([limit], |row| {
                Ok(LogEntry {
                    timestamp: row.get(0)?,
                    level: row.get(1)?,
                    message: row.get(2)?,
                })
            }) {
                for log in iter.flatten() {
                    logs.push(log);
                }
            }
        }
        logs
    }

    pub fn clear_logs(&self) {
        let _ = self.connection.execute("DELETE FROM logs", ());
    }

    pub fn set_state(&self, state: HashMap<String, String>) {
        for (key, value) in state {
            let _ = self.connection.execute(
                "INSERT OR REPLACE INTO config (key, value) VALUES(?1, ?2)",
                (&key, &value),
            );
        }
    }

    pub fn get_last_scan_time(&self) -> Option<String> {
        self.get_state().get("last_scan_time").cloned()
    }

    pub fn set_last_scan_time(&self, timestamp: String) {
        let _ = self.connection.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES('last_scan_time', ?1)",
            [&timestamp],
        );
    }

    pub fn store_photo(&self, photo: Photo) {
        let _ = self.connection.execute(
            "INSERT OR REPLACE INTO photo(id, location, encoded, latitude, longitude, created) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            (&photo.id, &photo.location, &photo.encoded, &photo.latitude, &photo.longitude, &photo.created),
        );
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

    pub fn update_photo_thumbnail(&self, id: &str, encoded: &str) {
        let _ = self
            .connection
            .execute("UPDATE photo SET encoded = ?1 WHERE id = ?2", (encoded, id));
    }

    pub fn list_objects(&self, query: &str) -> Vec<String> {
        let mut objects = Vec::new();
        let sql = "SELECT class FROM object WHERE class LIKE ?1 UNION SELECT value FROM properties WHERE (key LIKE '%City%' OR key LIKE '%Country%' OR key LIKE '%State%') AND value LIKE ?1 GROUP BY 1";
        if let Ok(mut stmt) = self.connection.prepare(sql) {
            if let Ok(iter) = stmt.query_map([format!("%{query}%")], |row| row.get(0)) {
                for item in iter.flatten() {
                    objects.push(item);
                }
            }
        }
        objects
    }

    pub fn list_photos(
        &self,
        query: &str,
        offset: usize,
        limit: usize,
        favorites_only: bool,
        videos_only: bool,
    ) -> Vec<Photo> {
        let mut photos = Vec::new();
        let fav_filter = if favorites_only {
            "AND EXISTS(SELECT 1 FROM properties WHERE photo_id=p.id AND key='favorite')"
        } else {
            ""
        };
        let video_filter = if videos_only {
            "AND (p.location LIKE '%.mp4' OR p.location LIKE '%.mkv' OR p.location LIKE '%.mov' OR p.location LIKE '%.avi' OR p.location LIKE '%.webm')"
        } else {
            ""
        };

        let is_uuid = query.len() == 36 && query.chars().all(|c| c.is_alphanumeric() || c == '-');

        let q_filter = if !query.is_empty() {
            if is_uuid {
                "AND (p.id = ?3 OR EXISTS(SELECT 1 FROM faces WHERE photo_id=p.id AND person_id = ?3))"
            } else {
                "AND (p.location LIKE ?3 OR p.id LIKE ?3 OR EXISTS(SELECT 1 FROM object WHERE photo_id=p.id AND class LIKE ?3) OR EXISTS(SELECT 1 FROM faces f JOIN people p_name ON f.person_id = p_name.id WHERE f.photo_id=p.id AND p_name.name LIKE ?3))"
            }
        } else {
            ""
        };

        let sql = format!("SELECT p.id, p.location, p.encoded, p.latitude, p.longitude, p.created, EXISTS(SELECT 1 FROM properties WHERE photo_id=p.id AND key='favorite') FROM photo p WHERE 1=1 {fav_filter} {video_filter} {q_filter} ORDER BY p.created DESC LIMIT ?1, ?2");
        if let Ok(mut stmt) = self.connection.prepare(&sql) {
            let q_param = if is_uuid {
                query.to_string()
            } else {
                format!("%{query}%")
            };
            let params: Vec<&dyn rusqlite::ToSql> = if !query.is_empty() {
                vec![&offset, &limit, &q_param]
            } else {
                vec![&offset, &limit]
            };
            if let Ok(iter) = stmt.query_map(params.as_slice(), |row| {
                Ok(Photo {
                    id: row.get(0)?,
                    location: row.get(1)?,
                    encoded: row.get(2)?,
                    created: row.get(5).unwrap_or_default(),
                    objects: HashMap::new(),
                    properties: HashMap::new(),
                    latitude: row.get(3).unwrap_or(0.0),
                    longitude: row.get(4).unwrap_or(0.0),
                    favorite: row.get(6).unwrap_or(false),
                })
            }) {
                for p in iter.flatten() {
                    photos.push(p);
                }
            }
        }
        photos
    }

    pub fn toggle_favorite(&self, photo_id: &str) -> bool {
        let exists = self
            .connection
            .query_row(
                "SELECT 1 FROM properties WHERE photo_id = ?1 AND key = 'favorite'",
                [photo_id],
                |_| Ok(true),
            )
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
        if let Ok(mut stmt) = self.connection.prepare("SELECT p.id, p.location, p.encoded, p.latitude, p.longitude, p.created, EXISTS(SELECT 1 FROM properties WHERE photo_id=p.id AND key='favorite') FROM photo p WHERE p.latitude != 0.0 OR p.longitude != 0.0") {
            if let Ok(iter) = stmt.query_map([], |row| {
                Ok(Photo {
                    id: row.get(0)?, location: row.get(1)?, encoded: row.get(2)?, created: row.get(5).unwrap_or_default(),
                    objects: HashMap::new(), properties: HashMap::new(), latitude: row.get(3).unwrap_or(0.0), longitude: row.get(4).unwrap_or(0.0), favorite: row.get(6).unwrap_or(false),
                })
            }) {
                for p in iter.flatten() { photos.push(p); }
            }
        }
        photos
    }

    pub fn store_face(&self, face: Face) {
        let embedding_bytes: Vec<u8> = face
            .embedding
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();
        let _ = self.connection.execute("INSERT OR REPLACE INTO faces(photo_id, face_id, crop_path, encoded, embedding, person_id) VALUES(?1, ?2, ?3, ?4, ?5, ?6)", (&face.photo_id, &face.face_id, &face.crop_path, &face.encoded, &embedding_bytes, &face.person_id));
    }

    pub fn get_people(&self) -> Vec<PersonWithFace> {
        let mut people = Vec::new();
        if let Ok(mut stmt) = self.connection.prepare("SELECT p.id, p.name, f.crop_path, f.face_id, f.encoded, p.embedding, (SELECT COUNT(*) FROM faces WHERE person_id = p.id) FROM people p LEFT JOIN faces f ON p.id = f.person_id WHERE p.name IS NOT NULL GROUP BY p.id") {
            if let Ok(iter) = stmt.query_map([], |row| {
                let embedding: Option<Vec<f32>> = row.get::<_, Option<Vec<u8>>>(5).ok().flatten().map(|bytes| bytes.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect());
                Ok(PersonWithFace { id: row.get(0)?, name: row.get(1)?, representative_crop: row.get(2).ok(), representative_face_id: row.get(3).ok(), encoded: row.get(4).ok(), embedding, face_count: row.get(6)? })
            }) {
                for p in iter.flatten() { people.push(p); }
            }
        }
        people
    }

    pub fn assign_name_to_face(&self, face_id: &str, name: &str) -> String {
        let existing_id: Option<String> = self
            .connection
            .query_row("SELECT id FROM people WHERE name = ?1", [name], |row| {
                row.get(0)
            })
            .ok();
        let current_person_id: Option<String> = self
            .connection
            .query_row(
                "SELECT person_id FROM faces WHERE face_id = ?1",
                [face_id],
                |row| row.get(0),
            )
            .ok();

        let target_id = match existing_id {
            Some(id) => {
                if let Some(anon_id) = current_person_id {
                    if anon_id != id {
                        let _ = self.connection.execute(
                            "UPDATE faces SET person_id = ?1 WHERE person_id = ?2",
                            (&id, &anon_id),
                        );
                        let _ = self
                            .connection
                            .execute("DELETE FROM people WHERE id = ?1", [&anon_id]);
                    }
                } else {
                    let _ = self.connection.execute(
                        "UPDATE faces SET person_id = ?1 WHERE face_id = ?2",
                        (&id, face_id),
                    );
                }
                id
            }
            None => {
                if let Some(id) = current_person_id {
                    let _ = self
                        .connection
                        .execute("UPDATE people SET name = ?1 WHERE id = ?2", (name, &id));
                    id
                } else {
                    let new_id = uuid::Uuid::new_v4().to_string();
                    let _ = self.connection.execute(
                        "INSERT INTO people (id, name) VALUES (?1, ?2)",
                        (&new_id, name),
                    );
                    let _ = self.connection.execute(
                        "UPDATE faces SET person_id = ?1 WHERE face_id = ?2",
                        (&new_id, face_id),
                    );
                    new_id
                }
            }
        };

        self.update_person_centroid(&target_id);
        target_id
    }

    pub fn create_anonymous_person(&self, embedding: &[f32]) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let embedding_bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        let _ = self.connection.execute(
            "INSERT INTO people (id, name, embedding) VALUES (?1, NULL, ?2)",
            (&id, &embedding_bytes),
        );
        id
    }

    pub fn update_person_centroid(&self, person_id: &str) {
        let mut embeddings = Vec::new();
        if let Ok(mut stmt) = self
            .connection
            .prepare("SELECT embedding FROM faces WHERE person_id = ?1")
        {
            if let Ok(rows) = stmt.query_map([person_id], |row| row.get::<_, Vec<u8>>(0)) {
                for row in rows.flatten() {
                    let emb: Vec<f32> = row
                        .chunks_exact(4)
                        .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
                        .collect();
                    if emb.len() == 512 {
                        embeddings.push(emb);
                    }
                }
            }
        }

        if embeddings.is_empty() {
            return;
        }

        let count = embeddings.len() as f32;
        let mut centroid = vec![0.0f32; 512];
        for emb in embeddings {
            for i in 0..512 {
                centroid[i] += emb[i];
            }
        }
        for i in 0..512 {
            centroid[i] /= count;
        }

        let norm: f32 = centroid.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in centroid.iter_mut() {
                *v /= norm;
            }
        }

        let centroid_bytes: Vec<u8> = centroid.iter().flat_map(|f| f.to_le_bytes()).collect();
        let _ = self.connection.execute(
            "UPDATE people SET embedding = ?1 WHERE id = ?2",
            (centroid_bytes, person_id),
        );
    }

    pub fn get_anonymous_people_groups(&self) -> Vec<PersonWithFace> {
        let mut results = Vec::new();
        if let Ok(mut stmt) = self.connection.prepare("SELECT p.id, f.crop_path, f.face_id, f.encoded, p.embedding, (SELECT COUNT(*) FROM faces WHERE person_id = p.id) FROM people p JOIN faces f ON p.id = f.person_id WHERE p.name IS NULL GROUP BY p.id ORDER BY (SELECT COUNT(*) FROM faces WHERE person_id = p.id) DESC") {
            if let Ok(iter) = stmt.query_map([], |row| {
                let embedding: Option<Vec<f32>> = row.get::<_, Option<Vec<u8>>>(4).ok().flatten().map(|bytes| bytes.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect());
                Ok(PersonWithFace {
                    id: row.get(0)?,
                    name: "Unnamed Person".to_string(),
                    representative_crop: row.get(1).ok(),
                    representative_face_id: row.get(2).ok(),
                    encoded: row.get(3).ok(),
                    embedding,
                    face_count: row.get(5)?
                })
            }) {
                for p in iter.flatten() { results.push(p); }
            }
        }
        results
    }

    pub fn get_faces_for_photo(&self, photo_id: &str) -> Vec<FaceWithPerson> {
        let mut faces = Vec::new();
        let sql = "SELECT f.photo_id, f.face_id, f.crop_path, f.encoded, f.person_id, p.name FROM faces f LEFT JOIN people p ON f.person_id = p.id WHERE f.photo_id = ?1";
        if let Ok(mut stmt) = self.connection.prepare(sql) {
            if let Ok(iter) = stmt.query_map([photo_id], |row| {
                Ok(FaceWithPerson {
                    photo_id: row.get(0)?,
                    face_id: row.get(1)?,
                    crop_path: row.get(2)?,
                    encoded: row.get(3)?,
                    person_id: row.get(4)?,
                    person_name: row.get(5)?,
                })
            }) {
                for f in iter.flatten() {
                    faces.push(f);
                }
            }
        }
        faces
    }

    pub fn get_person_faces(&self, person_id: &str) -> Vec<Face> {
        let mut faces = Vec::new();
        if let Ok(mut stmt) = self.connection.prepare("SELECT photo_id, face_id, crop_path, encoded, person_id FROM faces WHERE person_id = ?1") {
            if let Ok(iter) = stmt.query_map([person_id], |row| {
                Ok(Face {
                    photo_id: row.get(0)?,
                    face_id: row.get(1)?,
                    crop_path: row.get(2)?,
                    encoded: row.get(3)?,
                    embedding: Vec::new(), // Not needed for UI
                    person_id: row.get(4)?,
                })
            }) {
                for f in iter.flatten() { faces.push(f); }
            }
        }
        faces
    }

    pub fn get_photos_for_person(&self, person_id: &str) -> Vec<Photo> {
        let mut photos = Vec::new();
        if let Ok(mut stmt) = self.connection.prepare("SELECT p.id, p.location, p.encoded, p.latitude, p.longitude, p.created, EXISTS(SELECT 1 FROM properties WHERE photo_id=p.id AND key='favorite') FROM photo p JOIN faces f ON p.id = f.photo_id WHERE f.person_id = ?1 GROUP BY p.id") {
            if let Ok(iter) = stmt.query_map([person_id], |row| {
                Ok(Photo {
                    id: row.get(0)?, location: row.get(1)?, encoded: row.get(2)?, created: row.get(5).unwrap_or_default(),
                    objects: HashMap::new(), properties: HashMap::new(), latitude: row.get(3).unwrap_or(0.0), longitude: row.get(4).unwrap_or(0.0), favorite: row.get(6).unwrap_or(false),
                })
            }) {
                for p in iter.flatten() { photos.push(p); }
            }
        }
        photos
    }

    pub fn list_directories(&self) -> Vec<String> {
        let mut results = Vec::new();
        if let Ok(mut stm) = self.connection.prepare("SELECT name FROM directory") {
            if let Ok(iter) = stm.query_map((), |row| row.get(0)) {
                for val in iter.flatten() {
                    results.push(val);
                }
            }
        }
        results
    }

    pub fn remove_directory(&self, path: String) {
        let _ = self
            .connection
            .execute("DELETE FROM directory WHERE name = ?1", [&path]);
    }

    pub fn add_directory(&self, path: &str) {
        let _ = self
            .connection
            .execute("INSERT INTO directory (name) VALUES(?1)", [&path]);
    }

    pub fn merge_people(&self, from_id: &str, to_id: &str) {
        let _ = self.connection.execute(
            "UPDATE faces SET person_id = ?1 WHERE person_id = ?2",
            (to_id, from_id),
        );
        let _ = self
            .connection
            .execute("DELETE FROM people WHERE id = ?1", [from_id]);
        self.update_person_centroid(to_id);
    }

    pub fn rename_person(&self, id: &str, new_name: &str) {
        let _ = self
            .connection
            .execute("UPDATE people SET name = ?1 WHERE id = ?2", (new_name, id));
    }

    pub fn remove_directory_full(&self, path: &str) {
        let mut photo_ids = Vec::new();
        if let Ok(mut stmt) = self
            .connection
            .prepare("SELECT id FROM photo WHERE location LIKE ?1")
        {
            if let Ok(rows) = stmt.query_map([format!("{path}%")], |row| row.get::<_, String>(0)) {
                for id in rows.flatten() {
                    photo_ids.push(id);
                }
            }
        }
        for id in photo_ids {
            let _ = self
                .connection
                .execute("DELETE FROM object WHERE photo_id = ?1", [&id]);
            let _ = self
                .connection
                .execute("DELETE FROM faces WHERE photo_id = ?1", [&id]);
            let _ = self
                .connection
                .execute("DELETE FROM properties WHERE photo_id = ?1", [&id]);
            let _ = self
                .connection
                .execute("DELETE FROM photo WHERE id = ?1", [&id]);
        }
        let _ = self
            .connection
            .execute("DELETE FROM directory WHERE name = ?1", [path]);
    }

    pub fn path_exists(&self, path: &str) -> bool {
        self.connection
            .query_row("SELECT 1 FROM photo WHERE location = ?1", [path], |_| {
                Ok(true)
            })
            .unwrap_or(false)
    }

    pub fn import_photo(&self, id: &str, location: &str, created: &str) {
        let _ = self.connection.execute(
            "INSERT OR REPLACE INTO photo (id, location, created) VALUES (?1, ?2, ?3)",
            (id, location, created),
        );
    }

    pub fn get_media_counts(&self) -> (i64, i64) {
        let photo_count: i64 = self.connection.query_row("SELECT COUNT(*) FROM photo WHERE NOT (location LIKE '%.mp4' OR location LIKE '%.mkv' OR location LIKE '%.mov' OR location LIKE '%.avi' OR location LIKE '%.webm')", [], |r| r.get(0)).unwrap_or(0);
        let video_count: i64 = self.connection.query_row("SELECT COUNT(*) FROM photo WHERE (location LIKE '%.mp4' OR location LIKE '%.mkv' OR location LIKE '%.mov' OR location LIKE '%.avi' OR location LIKE '%.webm')", [], |r| r.get(0)).unwrap_or(0);
        (photo_count, video_count)
    }

    pub fn list_devices(&self) -> Vec<DeviceInfo> {
        let mut results = Vec::new();
        if let Ok(mut stmt) = self.connection.prepare("SELECT ip, name FROM device") {
            if let Ok(iter) = stmt.query_map([], |row| {
                Ok(DeviceInfo {
                    id: row.get::<_, String>(0)?, // Using IP as ID for now or it could be UUID
                    title: row.get::<_, String>(1)?,
                    icon: "mdi-cellphone".to_string(), // Default icon
                    up_to_date: true,
                    host: false,
                    photo_count: 0,
                    video_count: 0,
                    os: "unknown".to_string(),
                })
            }) {
                for d in iter.flatten() {
                    results.push(d);
                }
            }
        }
        results
    }

    pub fn get_all_people_with_embeddings(&self) -> Vec<(String, Vec<f32>)> {
        let mut results = Vec::new();
        if let Ok(mut stmt) = self
            .connection
            .prepare("SELECT id, embedding FROM people WHERE embedding IS NOT NULL")
        {
            if let Ok(iter) = stmt.query_map([], |row| {
                let id: String = row.get(0)?;
                let bytes: Vec<u8> = row.get(1)?;
                let emb: Vec<f32> = bytes
                    .chunks_exact(4)
                    .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
                    .collect();
                Ok((id, emb))
            }) {
                for p in iter.flatten() {
                    results.push(p);
                }
            }
        }
        results
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
    pub embedding: Vec<f32>,
    pub person_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PersonWithFace {
    pub id: String,
    pub name: String,
    pub representative_crop: Option<String>,
    pub representative_face_id: Option<String>,
    pub encoded: Option<String>,
    pub embedding: Option<Vec<f32>>,
    pub face_count: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub up_to_date: bool,
    pub host: bool,
    pub photo_count: i64,
    pub video_count: i64,
    pub os: String,
}
