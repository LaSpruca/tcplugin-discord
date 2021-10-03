use std::{collections::HashSet, env, sync::Arc};

use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
};
use tokio::sync::Mutex;
use tracing::{error, info};
use commands::*;
use crate::ws::WsManager;

mod commands;

pub struct WsManagerContainer;

impl TypeMapKey for WsManagerContainer {
    type Value = Arc<Mutex<WsManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        println!("Resumed");
    }
}

#[group]
#[commands(yaml)]
struct General;

pub async fn discord_setup(ws_mgr: Arc<Mutex<WsManager>>) {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    // Create the framework
    let framework = StandardFramework::new()
        .configure(|c| c
            .prefix("```")
            .dynamic_prefix(|_, msg| Box::pin(async move {
                Some("/".to_string())
            }))
            .ignore_bots(true)
        )
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<WsManagerContainer>(ws_mgr);
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
