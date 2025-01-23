// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_files,
            scan_files,
            listen_for_incomming_connect,
            list_devices,
            join_network,
            list_objects,
            get_thumbnail,
            get_ip,
            get_device_by_name
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn join_network(ip: String) -> String {
    server::request_offer(ip.to_string()).await
}

mod config;
mod database;
mod device;
mod file;
mod server;
mod transport;
#[tauri::command]
async fn listen_for_incomming_connect() {
    println!("Starting server");
    server::start().await;
}
use get_if_addrs::get_if_addrs;
use std::net::Ipv4Addr;
#[tauri::command()]
fn get_ip() -> String {
    let ifaces = get_if_addrs().unwrap();

    let mut ip = String::from("");
    for iface in ifaces {
        if let std::net::IpAddr::V4(ipv4) = iface.ip() {
            // Check if the IP address is within the local network ranges
            if is_local_network_ip(ipv4) {
                ip = ipv4.to_string();
                println!("Local Network IP: {}", ipv4);
                break; // Assuming you only need one IP, otherwise remove this line
            }
        }
    }
    ip
}
// Helper function to check if an IPv4 address is in the private range
fn is_local_network_ip(ip: Ipv4Addr) -> bool {
    ip.octets()[0] == 10
        || (ip.octets()[0] == 172 && (16..=31).contains(&ip.octets()[1]))
        || (ip.octets()[0] == 192 && ip.octets()[1] == 168)
}

#[tauri::command()]
fn list_devices() -> String {
    device::list_devices()
}
#[tauri::command]
fn list_objects(query: &str, path: &str) -> String {
    let database = database::Database::new(path);
    serde_json::to_string(&database.list_objects(query)).unwrap()
}

use serde::{Deserialize, Serialize};
use std::string::String;

#[derive(Serialize, Deserialize, Debug)]
struct Image {
    path: String,
    encoded: String,
}

#[tauri::command]
async fn list_files(
    path: String,
    query: String,
    limit: usize,
    offset: usize,
    scan: bool,
) -> String {
    if scan {
        println!("Scanning for photos.");
        scan_files(path.to_string(), path.clone());
    }
    let database = database::Database::new(&path);
    let photos = database.list_photos(&query, offset, limit);
    println!("{} photos retrieved", photos.len());
    serde_json::to_string(&photos).unwrap()
}

#[tauri::command]
fn scan_files(directory: String, path: String) {
    println!("Scanning folder {}", directory);
    file::scan_folder(directory, &path);
}

#[tauri::command]
fn get_thumbnail(path: String) -> String {
    println!("Generating thumnail for {}", path);
    file::get_thumbnail(path)
}

#[tauri::command]
fn get_device_by_name(name: String) -> String {
    device::get_device_by_name(name)
}
