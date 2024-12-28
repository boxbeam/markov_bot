use std::collections::HashMap;

use async_trait::async_trait;
use markov::MarkovChain;
use serenity::{
    all::{ChannelId, Context, EventHandler, GatewayIntents, Message, UserId},
    futures::lock::Mutex,
    Client,
};

mod markov;

#[derive(Default)]
struct MarkovCache {
    channels: HashMap<ChannelId, MarkovChain<5>>,
    members: HashMap<UserId, MarkovChain<5>>,
}

#[derive(Default)]
struct Handler(Mutex<MarkovCache>);

async fn send_message(ctx: &Context, channel: ChannelId, msg: String) {
    if let Err(err) = channel.say(&ctx.http, msg).await {
        eprintln!("Failed to send message: {err}");
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.guild_id.is_none() || msg.author.bot {
            return;
        }

        let mut cache = self.0.lock().await;

        if let Some(target) = msg.content.strip_prefix("usim ") {
            let target = target.replace("!", "").replace("<@", "").replace(">", "");
            let Ok(target): Result<u64, _> = target.parse() else {
                return;
            };
            let user = UserId::new(target);

            let Some(chain) = cache.members.get(&user) else {
                return;
            };

            let generated = chain.generate();
            send_message(&ctx, msg.channel_id, generated).await;
            return;
        }

        if msg.content.to_lowercase().starts_with("hi markov") {
            let Some(chain) = cache.channels.get(&msg.channel_id) else {
                return;
            };

            let generated = chain.generate();
            send_message(&ctx, msg.channel_id, generated).await;
            return;
        }

        let channels = cache.channels.entry(msg.channel_id).or_default();
        channels.digest(&msg.content);

        let chance = rand::random::<f64>();

        if chance < 0.005 {
            let generated = channels.generate();
            send_message(&ctx, msg.channel_id, generated).await;
        }

        cache
            .members
            .entry(msg.author.id)
            .or_default()
            .digest(&msg.content);
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler(Default::default()))
        .await
        .expect("Failed to create discord client");

    client.start().await.unwrap();
}
