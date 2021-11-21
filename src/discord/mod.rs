pub mod server_command;

use crate::{
    discord::server_command::ServerCommand,
    ws::{Am, WsManager},
};
use futures::stream::StreamExt;
use log::{debug, error, info};
use regex::Regex;
use std::{env, error::Error, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_embed_builder::{EmbedAuthorBuilder, EmbedBuilder, EmbedError, EmbedFieldBuilder};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_http::Client as HttpClient;
use twilight_model::{
    channel::embed::{Embed, EmbedField},
    gateway::Intents,
};

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

    // Process each event as they come in.
    while let Some((shard_id, event)) = events.next().await {
        // Update the cache with the event.
        cache.update(&event);

        let mgr2 = ws_mgr.clone();

        tokio::spawn(handle_event(shard_id, event, Arc::clone(&http), mgr2));
    }

    Ok(())
}

pub fn create_error_embed(title: &str, error: &str) -> Result<Embed, EmbedError> {
    EmbedBuilder::new()
        .color(0xda2b46)
        .title(&format!(":x: {}", title))
        .field(EmbedFieldBuilder::new("Error", error).build())
        .build()
}

pub fn create_embed(
    title: &str,
    author: Option<&str>,
    fields: Vec<EmbedField>,
) -> Result<Embed, EmbedError> {
    Ok(Embed {
        fields,
        ..if let Some(author) = author {
            EmbedBuilder::new()
                .color(0x78b064)
                .title(title)
                .author(EmbedAuthorBuilder::new().name(author).build())
                .build()?
        } else {
            EmbedBuilder::new().color(0x78b064).title(title).build()?
        }
    })
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
            info!("Got server command");

            let command = msg
                .content
                .strip_prefix("```yaml\n")
                .unwrap()
                .strip_suffix("```")
                .unwrap();
            let executable: ServerCommand = match serde_yaml::from_str(command) {
                Ok(a) => a,
                Err(e) => {
                    http.create_message(msg.channel_id)
                        .embeds(&[create_error_embed(
                            "Error parsing command",
                            &format!("{}", e),
                        )?])?
                        .exec()
                        .await?;
                    return Ok(());
                }
            };

            let server_selector = match Regex::new(&executable.on) {
                Ok(server_selector) => {
                    ws_mgr
                        .lock()
                        .await
                        .get_connections_by_regex(
                            server_selector,
                            msg.channel_id.to_string(),
                        )
                        .await
                }
                Err(_) => {
                    match ws_mgr
                        .lock()
                        .await
                        .get_connection_by_name(
                            executable.on.clone(),
                            msg.channel_id.to_string(),
                        )
                        .await
                    {
                        Some(x) => vec![x],
                        None => vec![],
                    }
                }
            };

            info!("{}", server_selector.len());

            if server_selector.is_empty() {
                debug!("No servers found");
                http.create_message(msg.channel_id)
                    .embeds(&[create_error_embed(
                        "Could not find any servers",
                        &format!("No servers matched the query {}", &executable.on),
                    )?])
                    .unwrap()
                    .exec()
                    .await?;
                return Ok(());
            }

            for server in server_selector {
                debug!("Sending to {}", server.0);
                match server
                    .1
                    .lock()
                    .await
                    .send_server_command(executable.clone())
                    .await {
                    Err(err) => {
                        error!("Error sending packet to {}, {}", server.0, err.to_string())
                    }
                    _ => {}
                };
            }
        }
        // Global command (does not affect 1 server)
        Event::MessageCreate(msg) if msg.content.starts_with("/") => {
            let command = msg
                .content
                .strip_prefix("/")
                .unwrap()
                .split(" ")
                .next()
                .unwrap_or(msg.content.strip_prefix("/").unwrap());
            if command == "list" {
                let mut fields = vec![];
                for (uuid, k) in ws_mgr
                    .lock()
                    .await
                    .get_connected_by_ctrl_channel_id(msg.channel_id.to_string())
                    .await
                {
                    fields.push(
                        EmbedFieldBuilder::new(k.lock().await.get_name(), uuid.to_string()).build(),
                    );
                }
                http.create_message(msg.channel_id)
                    .embeds(&[Embed {
                        fields,
                        ..EmbedBuilder::new().title("Active Server").build().unwrap()
                    }])
                    .unwrap()
                    .exec()
                    .await
                    .unwrap();
            }
        }

        Event::ShardConnected(_) => {
            info!(
                "Connected on shard {} as {}",
                shard_id,
                http.current_user_application()
                    .exec()
                    .await
                    .unwrap()
                    .model()
                    .await
                    .unwrap()
                    .name
            );
            ws_mgr.lock().await.set_get_http(http.clone()).await;
        }
        _ => {}
    }

    Ok(())
}
