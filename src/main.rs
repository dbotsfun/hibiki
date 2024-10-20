use std::env;
use std::sync::Arc;
use tokio::task::JoinSet;
use twilight_gateway::{create_recommended, Config, EventTypeFlags, StreamExt};
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use twilight_model::gateway::presence::ActivityType;
use twilight_model::gateway::Intents;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::marker::RoleMarker;
use twilight_model::id::Id;
use vesper::prelude::*;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("No DISCORD_TOKEN found");
    let application_id = env::var("DISCORD_APPLICATION_ID")
        .unwrap()
        .parse::<u64>()
        .expect("No app id found");

    let http_client = Arc::new(Client::new(token.clone()));

    let config = Config::new(token.clone(), Intents::GUILD_PRESENCES);
    let shards = create_recommended(&http_client, config, |_, builder| builder.build())
        .await
        .unwrap()
        .collect::<Vec<_>>();

    let framework = Arc::new(
        Framework::builder(http_client.clone(), Id::new(application_id), ())
            .command(hello)
            .build(),
    );

    let mut set = JoinSet::new();
    for mut shard in shards {
        let framework = Arc::clone(&framework);
        set.spawn(async move {
            while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
                let Ok(event) = item else {
                    eprintln!("error receiving event: {:?}", item.unwrap_err());
                    continue;
                };
                match event {
                    Event::Ready(_) => {
                        println!("Ready");
                        framework.register_global_commands().await.unwrap();
                    }
                    Event::InteractionCreate(interaction) => {
                        framework.process(interaction.0).await;
                    }
                    Event::MemberAdd(member) => {
                        framework
                            .http_client()
                            .add_guild_member_role(
                                member.guild_id,
                                member.user.id,
                                Id::<RoleMarker>::new(1201246584382439475u64),
                            )
                            .await
                            .unwrap();
                    }
                    Event::PresenceUpdate(presence) => {
                        let custom_activity = presence.activities.iter().any(|a| {
                            a.kind == ActivityType::Custom
                                && a.state
                                    .as_ref()
                                    .map_or(false, |state| state.contains("dbots.fun"))
                        });

                        if custom_activity {
                            println!("Added role to user");
                            framework
                                .http_client()
                                .add_guild_member_role(
                                    presence.guild_id,
                                    presence.user.id(),
                                    Id::<RoleMarker>::new(1297596217198510210u64),
                                )
                                .await
                                .unwrap();
                        } else {
                            let has_role = framework
                                .http_client()
                                .guild_member(presence.guild_id, presence.user.id())
                                .await
                                .unwrap()
                                .model()
                                .await
                                .unwrap()
                                .roles
                                .contains(&Id::<RoleMarker>::new(1297596217198510210u64));

                            if has_role {
                                framework
                                    .http_client()
                                    .remove_guild_member_role(
                                        presence.guild_id,
                                        presence.user.id(),
                                        Id::<RoleMarker>::new(1297596217198510210u64),
                                    )
                                    .await
                                    .unwrap();
                                println!("Removed role from user");
                            }
                        }
                    }
                    _ => (),
                }
            }
        });
    }

    while set.join_next().await.is_some() {}
}

#[command]
#[description("Hewwo!")]
async fn hello(ctx: &SlashContext<()>) -> DefaultCommandResult {
    ctx.interaction_client
        .create_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some(String::from("Hello!")),
                    ..Default::default()
                }),
            },
        )
        .await?;

    Ok(())
}
