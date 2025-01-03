use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use serde::Serialize;

use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use crate::database::{Database};
use crate::transport::generate_offer;

fn handle_read(mut stream: &TcpStream) -> Option<Device> {
    let mut buf = [0u8; 4096];
    stream.read(&mut buf).unwrap();
    let req_str = String::from_utf8_lossy(&buf);
    let mut iterator = req_str.lines();
    let line = iterator.next().unwrap();

    if line.starts_with("GET /new-device HTTP/1.1") {
        println!("New device");
        let ip = stream.peer_addr().unwrap().ip();
        let device = Device {
            ip: ip.to_string(),
            name: "Arch".to_string(),
            offer: String::from(""),
        };
        Some(device)
    } else {
        None
    }
}
#[derive(Debug, Clone)]
pub struct Devices {
    list: Vec<Device>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Device {
    pub ip: String,
    pub name: String,
    pub offer: String,
}

#[derive(Serialize)]
struct offer {
    SDP: RTCSessionDescription,
}
async fn handle_write(mut stream: TcpStream, device: Device) {
    let sdp = generate_offer().await.unwrap();

    let offer = offer { SDP: sdp };

    let json = serde_json::to_string(&offer).unwrap();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n {} \r\n",
        json
    );

    let device = Device {
        ip: stream.peer_addr().unwrap().ip().to_string(),
        name: String::from("Jhonny"),
        offer: json,
    };

    let database = Database::new("/home/denzyl/");
    database.add_device(&device);
    match stream.write(response.as_bytes()) {
        Ok(_) => println!("Response sent"),
        Err(e) => println!("Failed sending response: {}", e),
    }
}

async fn handle_client(stream: TcpStream) {
    let device = handle_read(&stream);
    match device {
        Some(device) => {
            handle_write(stream, device).await;
        }
        None => {
            println!("No device found");
        }
    }
}

pub fn list_all_connected_devices() -> Vec<Device> {
    unsafe { DEVICES.clone() }
}
pub static mut DEVICES: Vec<Device> = Vec::new();
pub async fn start() {
    let listener = TcpListener::bind("0.0.0.0:9489").unwrap();
    println!("Listening for connections on port {}", 9489);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Incomming connection");
                handle_client(stream).await
            }
            Err(e) => {
                println!("Unable to connect: {}", e);
            }
        }
    }
}

pub async fn request_offer(ip: String) -> String {
    println!("{ip}");

    "Request made".to_string()
}
