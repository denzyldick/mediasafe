use std::{
    collections::HashMap,
    fs::{self, File},
};

use rusqlite::Connection;
use serde::Serialize;

use crate::server::Device;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn new(cache_path: &str) -> Self {
        let path = format!("{}/database.sql", cache_path);
        println!("Database.sql location: {}", path);
        let file = fs::metadata(&path);
        if let Err(kind) = file {
            File::create(&path);
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
            (),
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

        conn.execute(
            "
CREATE TABLE IF NOT EXISTS device(
        ip STRING,
        name STRING,
        offer STRING
)
",
            (),
        );
        Self { connection: conn }
    }

    pub fn add_device(&self, device: &Device) {
        let result = self.connection.execute(
            "INSERT INTO device(ip, name, offer) VALUES(?1,?2, ?3)",
            (&device.ip, &device.name, &device.offer),
        );

        if let Ok(result) = result {
            println!("Device has been stored");
        }

        if let Err(error) = result {
            println!("{}", error);
        }
    }
    pub fn store_photo(&self, photo: Photo) {
        let result = self.connection.execute(
            "INSERT INTO photo(id, location, encoded)
        VALUES(?1, ?2, ?3 )
",
            (&photo.id, &photo.location, &photo.encoded),
        );

        if let Ok(result) = result {
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
    pub fn get_photo(self, id: &str) {
        let sql = "select id,encoded from photo where photo.id = :id";
        let mut stmt = self.connection.prepare(sql).unwrap();

        stmt.query_map(&[(":id", &"one")], |row| {
            let var_name = Ok(String::from("Hfoaufaea"));

            var_name
        })
        .unwrap();
    }
    pub fn list_photos(self, query: &str, offset: usize, limit: usize) -> Vec<Photo> {
        let mut photos = Vec::new();
        if !query.is_empty() {
            let   sql = "select * from photo LEFT JOIN object  ON photo.id = object.photo_id WHERE class LIKE ?3 ORDER by object.probability DESC LIMIT ?1, ?2;";
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
            let sql = "SELECT id, location, encoded FROM photo LIMIT ?1, ?2";
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
        &println!("Photos found, {}, {} {}", photos.len(), offset, limit);
        photos
    }
    pub fn get_device_by_name(&self, name: String) -> Option<Device> {
        let sql = "SELECT name, offer, ip FROM device WHERE name = ?1";

        let params = &[(&name)];
        let mut stm = self.connection.prepare(sql).unwrap();
        let devices = stm
            .query_map(params, |row| {
                Ok(Device {
                    ip: row.get(2)?,
                    name: row.get(0)?,
                    offer: row.get(1)?,
                })
            })
            .unwrap();

        for device in devices {
            return Some(device.unwrap());
        }
        None
    }

    pub(crate) fn list_devices(&self) -> Vec<Device> {
        let sql = "SELECT ip, name, offer FROM device";
        let mut statement = self.connection.prepare(sql).unwrap();
        let device_iter = statement.query_map((), |device| {
            Ok(Device {
                ip: device.get(0)?,
                name: device.get(1)?,
                offer: device.get(2)?,
            })
        });

        let mut devices = Vec::new();

        for d in device_iter.unwrap() {
            devices.push(d.unwrap());
        }
        devices
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
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView};
use tch::{nn, vision::imagenet, Device as DeviceTch, Tensor};

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

mod tests {
    

    #[test]
    fn add_device() {
        let device = Device {
            ip: String::new(),
            name: String::from("test"),
            offer: String::new(),
        };

        let database = Database::new("./tests");
        database.add_device(&device);

        let d = database.get_device_by_name(device.name.clone()).unwrap();

        assert_eq!(device.name, d.name)
    }

    #[test]
    fn clasify_image() {
        let photo = Photo {
            id: String::from("1"),
            location: String::from(""),
            encoded: String::from(""),
            objects: HashMap::new(),
            properties: HashMap::new(),
        };
    }
}
