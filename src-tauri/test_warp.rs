use warp::Filter;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let route = warp::path("media").and(warp::fs::dir("/"));
    
    let (addr, server) = warp::serve(route).bind_ephemeral(([127, 0, 0, 1], 0));
    println!("Server bound to {}", addr.port());
    
    // Test a request
    let resp = reqwest::get(format!("http://127.0.0.1:{}/media/home/denzyl/Projects/mediasafe/package.json", addr.port())).await.unwrap();
    println!("Status: {}", resp.status());
    println!("Body: {}", resp.text().await.unwrap());
}
