use crate::{database, server::Device};

/// This function will add a device to the database
pub(crate) fn get_device_by_name(name: &str) -> Device {
    let database = database::Database::new("/home/denzyl/");
    

    database.get_device_by_name(name).unwrap()
}

/// This list all the deviecis in the database
pub(crate) fn list_devices() -> Vec<Device> {
    let database = database::Database::new("/home/denzyl/");

    let devices: Vec<Device> = database.list_devices();

    println!("Found {} devices. ", devices.len());

    devices
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
        assert_eq!(device_retrieved.name, device.name);
    }

    pub fn tests_list_devices() {
        let first_device = Device {
            name: "first_device".to_string(),
            ip: "".to_string(),
            offer: "Offer".to_string(),
        };

        let second_deveice = Device {
            name: "second_device".to_string(),
            ip: "".to_string(),
            offer: "Offer".to_string(),
        };

        database::Database::new(".tests/database.sql").add_device(&first_device);
        database::Database::new(".tests/database.sql").add_device(&second_deveice);

        let devices = list_devices();

        for device in devices {
            assert_eq!(device.name, "first_device");
            assert_eq!(device.name, "second_device");
        }
    }
}
