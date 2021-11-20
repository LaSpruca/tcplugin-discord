mod server_command;

use std::{env, error::Error, sync::Arc};
use futures::stream::StreamExt;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{cluster::{Cluster, ShardScheme}, Event};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;
use twilight_model::id::GuildId;
use log::{info, debug};
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};
use twilight_model::channel::embed::{Embed, EmbedAuthor, EmbedField};
use crate::discord::server_command::ServerCommand;
use crate::ws::{Am, WsManager};

pub async fn main(ws_mgr: Am<WsManager>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let token = env::var("DISCORD_TOKEN")?;

    // This is the default scheme. It will automatically create as many
    // shards as is suggested by Discord.
    let scheme = ShardScheme::Auto;

    // Use intents to only receive guild message events.
    let (cluster, mut events) = Cluster::builder(token.to_owned(), Intents::GUILD_MESSAGES)
        .shard_scheme(scheme)
        .build()
        .await?;
    let cluster = Arc::new(cluster);

    // Start up the cluster.
    let cluster_spawn = Arc::clone(&cluster);
    let cluster_spawn2 = Arc::clone(&cluster);
    // Start all shards in the cluster in the background.
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        cluster_spawn2.down()
    });

    // HTTP is separate from the gateway, so create a new client.
    let http = Arc::new(HttpClient::new(token));

    // Since we only care about new messages, make the cache only
    // cache new messages.
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    http.guild_members(GuildId::new(632402187112153088).unwrap());

    // Process each event as they come in.
    while let Some((shard_id, event)) = events.next().await {
        // Update the cache with the event.
        cache.update(&event);

        let mgr2 = ws_mgr.clone();

        tokio::spawn(handle_event(shard_id, event, Arc::clone(&http), mgr2));
    }

    Ok(())
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: Arc<HttpClient>,
    ws_mgr: Am<WsManager>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        // Server control commands
        Event::MessageCreate(msg) if msg.content.starts_with("```yaml") => {
            let command = msg.content.strip_prefix("```yaml\n").unwrap().strip_suffix("```").unwrap();
            let executable: ServerCommand = match serde_yaml::from_str(command) {
                Ok(a) => a,
                Err(e) => {
                    http.create_message(msg.channel_id).content(&format!(":x: Error running command: \n{}", e));
                    return Ok(());
                }
            };
            debug!("{:?}", executable);
        }
        // Global command (does not affect 1 server)
        Event::MessageCreate(msg) if msg.content.starts_with("/") => {
            let command = msg.content.strip_prefix("/").unwrap().split(" ").next().unwrap_or(msg.content.strip_prefix("/").unwrap());
            if command == "list" {
                let mut fields = vec![];
                for (uuid, k) in ws_mgr.lock().await.get_connected_by_guild(msg.guild_id.unwrap().to_string()).await {
                    fields.push(EmbedFieldBuilder::new(k.lock().await.get_name(), uuid.to_string()).build());
                }
                http
                    .create_message(msg.channel_id)
                    .embeds(&[Embed { fields, ..EmbedBuilder::new().title("Active Server").build().unwrap() }])
                    .unwrap()
                    .exec()
                    .await
                    .unwrap();
            }
        }

        Event::ShardConnected(_) => {
            info!("Connected on shard {} as {}", shard_id,
                http
                .current_user_application()
                .exec()
                .await
                .unwrap()
                .model()
                .await
                .unwrap()
                .name);
        }
        _ => {}
    }

    Ok(())
}