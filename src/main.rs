mod commands;
mod framework;
mod handlers;

use framework::build_framework;
use std::env;
use std::sync::Arc;
use tokio::task::JoinSet;
use twilight_gateway::{create_recommended, Config};
use twilight_http::Client;
use twilight_model::gateway::Intents;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("No DISCORD_TOKEN found");
    let application_id = env::var("DISCORD_APPLICATION_ID")
        .unwrap()
        .parse::<u64>()
        .expect("No app id found");

    let http_client = Arc::new(Client::new(token.clone()));

    let config = Config::new(
        token.clone(),
        Intents::GUILD_PRESENCES | Intents::GUILD_MEMBERS,
    );
    let shards = create_recommended(&http_client, config, |_, builder| builder.build())
        .await
        .unwrap()
        .collect::<Vec<_>>();

    let framework = build_framework(http_client.clone(), application_id);

    let mut set = JoinSet::new();
    for mut shard in shards {
        let framework = Arc::clone(&framework);
        set.spawn(async move {
            handlers::handle_shard_events(&mut shard, framework).await;
        });
    }

    while set.join_next().await.is_some() {}
}
