use crate::{database, server::Device};

/// This function will add a device to the database
pub(crate) fn get_device_by_name(name: String) -> String {
    let database = database::Database::new("/home/denzyl/");
    let device = database.get_device_by_name(name).unwrap();

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
