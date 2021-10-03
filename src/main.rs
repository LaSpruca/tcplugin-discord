use std::sync::Arc;
use tokio::sync::Mutex;
use crate::discord::discord_setup;

mod ws;
mod discord;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt::init();

    let manager = ws::WsManager::new().await;

    discord_setup(Arc::new(Mutex::new(manager))).await;
}