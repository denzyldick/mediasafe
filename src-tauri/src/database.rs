use std::{
    collections::HashMap,
    fs::{self, File},
};

use rusqlite::Connection;
use serde::Serialize;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn new(cache_path: &str) -> Self {
        let path = format!("{}/database.sql", cache_path);
        println!("Database.sql location: {}", path);
        let file = fs::metadata(&path);
        match file {
            Err(kind) => {
                File::create(&path);
            }
            _ => {}
        }
        let conn = Connection::open(format!("{}/database.sql", cache_path)).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS photo (
            id    STRING PRIMARY KEY,
            location  STRING,
            encoded STRING,
            created DATE_TIME
        )
",
            (), // empty list of parameters.
        )
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
        );
        Self { connection: conn }
    }

    pub fn store_photo(&self, photo: Photo) {
        self.connection.execute(
            "INSERT INTO photo(id, location, encoded)
        VALUES(?1, ?2, ?3)
",
            (&photo.id, &photo.location, &photo.encoded),
        );

        println!("Photo has been saved");
        for (object, probablity) in photo.objects {
            self.connection
                .execute(
                    "INSERT INTO object (photo_id, class, probability) VALUES(?1, ?2, ?3)",
                    (&photo.id, &object, &probablity),
                )
                .unwrap();

            println!("Inserting probability");
        }

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
    pub fn list_photos(self, query: &str, offset: usize, limit: usize) -> Vec<Photo> {
        let mut photos = Vec::new();
        if !query.is_empty() {
            let   sql = "select * from photo JOIN object  ON photo.id = object.photo_id WHERE class LIKE ?3 ORDER by object.probability DESC LIMIT ?1, ?2;";
            let param = (offset.to_string(), limit.to_string(), format!("%{query}"));
            let mut stmt = self.connection.prepare(sql).unwrap();
            let person_iter = stmt
                .query_map(param, |row| {
                    Ok(Photo {
                        id: row.get(0)?,
                        location: row.get(1)?,
                        encoded: row.get(2)?,
                        objects: HashMap::new(),
                        properties: HashMap::new(),
                    })
                })
                .unwrap();
            for p in person_iter {
                photos.push(p.unwrap());
            }
        } else {
            let sql = "SELECT id, location, encoded FROM photo JOIN properties ON  photo.id = properties.photo_id WHERE properties.key = 'DateTimeOriginal' ORDER BY properties.value DESC LIMIT ?1, ?2";
            let param = (offset.to_string(), limit.to_string());

            let mut stmt = self.connection.prepare(sql).unwrap();
            let person_iter = stmt
                .query_map(param, |row| {
                    Ok(Photo {
                        id: row.get(0)?,
                        location: row.get(1)?,
                        encoded: row.get(2)?,
                        objects: HashMap::new(),
                        properties: HashMap::new(),
                    })
                })
                .unwrap();
            for p in person_iter {
                photos.push(p.unwrap());
            }
        }
        println!("Photos found, {}, {} {}", photos.len(), offset, limit);
        photos
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Photo {
    pub id: String,
    pub location: String,
    pub encoded: String,
    pub objects: HashMap<String, f64>,
    pub properties: HashMap<String, String>,
}
