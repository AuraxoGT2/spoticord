use std::env;
use std::sync::LazyLock;

pub static DISCORD_TOKEN: LazyLock<String> = LazyLock::new(|| {
    std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN environment variable")
});
pub static DATABASE_URL: LazyLock<String> = LazyLock::new(|| {
    std::env::var("DATABASE_URL").expect("missing DATABASE_URL environment variable")
});
pub static LINK_URL: LazyLock<String> = LazyLock::new(|| {
    std::env::var("LINK_URL").expect("missing LINK_URL environment variable")
});
pub static SPOTIFY_CLIENT_ID: LazyLock<String> = LazyLock::new(|| {
    std::env::var("SPOTIFY_CLIENT_ID").expect("missing SPOTIFY_CLIENT_ID environment variable")
});
pub static SPOTIFY_CLIENT_SECRET: LazyLock<String> = LazyLock::new(|| {
    std::env::var("SPOTIFY_CLIENT_SECRET")
        .expect("missing SPOTIFY_CLIENT_SECRET environment variable")
});

// Locked behind `stats` feature
pub static KV_URL: LazyLock<String> = LazyLock::new(|| {
    std::env::var("KV_URL").expect("missing KV_URL environment variable")
});

use std::net::SocketAddr;
use warp::Filter; // assuming you're using warp for HTTP server

#[tokio::main]
async fn main() {
    // Get the port from the environment variable, default to 10000 if not set
    let port = env::var("PORT").unwrap_or_else(|_| "10000".to_string());
    let port: u16 = port.parse().unwrap_or(10000);

    // Create the address to bind to
    let addr: SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .expect("Invalid port");

    // Example: Set up a simple server with warp (change based on your server setup)
    let routes = warp::any().map(|| "Hello, World!");

    // Start the server
    warp::serve(routes)
        .run(addr)
        .await;

    println!("Server running on http://127.0.0.1:{}", port);
}
