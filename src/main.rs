pub mod db;

use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::Client;
use serenity::model::channel::Message;
use serenity::framework::standard::CommandResult;
use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::prelude::*;
use crate::db::Db;
use std::sync::Arc;

#[group]
#[commands(vote_help, poll)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler{}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);

    let token = std::env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Failed to create client.");

    {
        let mut data = client.data.write().await;
        data.insert::<Db>(Arc::new(Db::new(&std::env::var("DATABASE_URL").unwrap()).await.unwrap()))
    }

    client.start().await.unwrap();
}

#[command]
async fn vote_help(ctx: &Context, msg: &Message) -> CommandResult{
    msg.reply_ping(ctx, r#"
Ranked Polls Discord Bot Help:
 - !vote_help, show this help message
 - !poll, show current poll options
 - !vote <1st choice> <2nd choice> … <nth choice>, cast or update your vote run !poll to see choice IDs
 - !tally, show the results thus far
    "#).await?;

    Ok(())
}

#[command]
async fn poll(ctx: &Context, msg: &Message) -> CommandResult{
    let data = ctx.data.read().await;
    let db = data.get::<Db>().unwrap();

    let mut reply = String::new();
    reply.push_str("The options in the current poll are as follows:\n");

    let candidates = db.list_candidates().await?;
    for candidate in candidates{
        reply.push_str(&format!(" - {} (ID is {})\n", candidate.name, candidate.id));
    }

    reply.push_str("\nTo vote, run !vote and list the IDs of your candidates from most preferable to least preferable. You do not need to list all candidates. For example, if you like the candidate with ID 3 best, then the candidate with ID 1, and then like the candidate with ID 2 the least, you would run `!vote 3 1 2`.");

    msg.reply_ping(ctx, reply).await?;

    Ok(())
}
