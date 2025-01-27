use crate::{database, server::Device};

/// This function will add a device to the database
pub(crate) fn get_device_by_name(name: &str) -> String {
    let database = database::Database::new("/home/denzyl/");
    let device = database.get_device_by_name(&name).unwrap();

    println!("Ask for Device = {} ", device.name);
    serde_json::to_string(&device).unwrap()
}

/// This list all the deviecis in the database
pub(crate) fn list_devices() -> String {
    let database = database::Database::new("/home/denzyl/");

    let devices: Vec<Device> = database.list_devices();

    println!("Found {} devices. ", devices.len());

    serde_json::to_string(&devices).unwrap()
}

mod tests {
    use crate::{
        database,
        device::{get_device_by_name, list_devices},
        server::Device,
    };

    pub fn test_get_device_by_name() {
        let device = Device {
            name: "test".to_string(),
            ip: "192.168.1.1".to_string(),
            offer: String::from("offer"),
        };
        database::Database::new(".tests/database.sql").add_device(&device);
        let device_retrieved = get_device_by_name("test");
        assert_eq!(device_retrieved, serde_json::to_string(&device).unwrap());
    }
}
