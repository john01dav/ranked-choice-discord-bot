use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::Client;
use serenity::model::channel::Message;
use serenity::framework::standard::CommandResult;
use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::prelude::*;

#[group]
#[commands(vote_help)]
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

    client.start().await.unwrap();
}

#[command]
async fn vote_help(ctx: &Context, msg: &Message) -> CommandResult{
    msg.reply(ctx, r#"
Ranked Polls Discord Bot Help:
 - !vote_help, show this help message
 - !poll, show current poll options with numbers
 - !vote <1st choice> <2nd choice> … <nth choice>, cast or update your vote
 - !tally, show the results thus far
    "#).await?;

    msg.author.id

    Ok(())
}

