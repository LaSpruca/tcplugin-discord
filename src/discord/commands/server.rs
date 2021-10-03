use serenity::{
    framework::standard::{macros::command, CommandResult, CommandError},
    model::prelude::*,
    prelude::*
};
use anyhow::anyhow;
use crate::discord::commands::server_command::ServerCommand;

#[command]
pub async fn yaml(ctx: &Context, msg: &Message) -> CommandResult {
    println!("Got message!");
    let content = msg.content.clone();

    if !content.starts_with("```yaml") {
        msg.reply(ctx, "Please use ```yaml for server commands").await;
        return CommandResult::Err(CommandError::from(anyhow!("Ya fucked up")));
    }

    let yaml_document = content.strip_prefix("```yaml\n").unwrap().strip_suffix("```").unwrap();

    let serialized: ServerCommand = serde_yaml::from_str(yaml_document).unwrap();

    println!("{:?}", serialized);

    Ok(())
}