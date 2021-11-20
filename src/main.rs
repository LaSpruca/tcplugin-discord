use std::sync::Arc;
use tokio::sync::Mutex;

mod ws;
mod discord;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let manager = ws::WsManager::new().await;

    discord::main(Arc::new(Mutex::new(manager))).await.unwrap();
}