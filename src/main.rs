pub mod db;
pub mod tally;

use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::Client;
use serenity::model::channel::Message;
use serenity::framework::standard::{CommandResult, Args};
use serenity::client::{Context, EventHandler};
use serenity::framework::standard::macros::{command, group};
use crate::db::Db;
use std::sync::Arc;

#[group]
#[commands(vote_help, poll, vote, tally)]
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

macro_rules! command_wrapper{
    ($name: tt, $internal: path) => {
        #[command]
        async fn $name(ctx: &Context, msg: &Message, args: Args) -> CommandResult{
            match $internal(ctx, msg, args).await {
                Ok(a) => Ok(a),
                Err(err) => {
                    msg.reply_ping(ctx, &format!("Failed: {}", err)).await?;
                    Err(err)
                }
            }
        }
    }
}

command_wrapper!(vote_help, vote_help_internal);

async fn vote_help_internal(ctx: &Context, msg: &Message, _args: Args) -> CommandResult{
    msg.reply_ping(ctx, r#"
Ranked Polls Discord Bot Help:
 - !vote_help, show this help message
 - !poll, show current poll options
 - !vote <1st choice> <2nd choice> … <nth choice>, cast or update your vote run !poll to see choice IDs
 - !tally, show the results thus far
    "#).await?;

    Ok(())
}

command_wrapper!(poll, poll_internal);

async fn poll_internal(ctx: &Context, msg: &Message, _args: Args) -> CommandResult{
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

command_wrapper!(vote, vote_internal);

async fn vote_internal(ctx: &Context, msg: &Message, mut args_unusable: Args) -> CommandResult{
    let data = ctx.data.read().await;
    let db = data.get::<Db>().unwrap();

    let acceptable_ids = db.list_candidates().await?.into_iter().map(|c| c.id).collect::<Vec<u32>>();

    let mut args = Vec::new();
    for arg in args_unusable.iter::<u32>(){
        let id = arg?;
        if !acceptable_ids.contains(&id){
            msg.reply_ping(ctx, &format!("{} is not a valid ID. Run !poll to see a list of options and valid IDs. You vote has not been saved.", id)).await?;
            return Ok(());
        }

        args.push(id);
    }

    db.set_vote(msg.author.id.0, args).await?;

    msg.reply_ping(ctx, "Congratulations! Your vote has been saved.").await?;

    Ok(())
}

command_wrapper!(tally, tally_internal);

async fn tally_internal(ctx: &Context, msg: &Message, _args: Args) -> CommandResult{
    let data = ctx.data.read().await;
    let db = data.get::<Db>().unwrap();
    let results = tally::tally(db.as_ref()).await?;
    msg.reply_ping(ctx, results).await?;
    Ok(())
}