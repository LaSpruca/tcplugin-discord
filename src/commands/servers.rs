use serenity::framework::standard::{CommandResult, macros::command};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
pub async fn yaml(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let author = msg.member.clone().unwrap();

    println!("{:?}", author.roles);

    println!("{}", msg.content);

    msg.channel_id.say(&ctx.http, "Jamer").await.unwrap();

    Ok(())
}