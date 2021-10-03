mod ws;

#[tokio::main]
async fn main() {
    let manager = ws::WsManager::new().await;

    loop {}
}