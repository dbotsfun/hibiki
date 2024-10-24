use std::sync::Arc;
use tokio::time::sleep;
use twilight_gateway::{Event, EventTypeFlags, Shard, StreamExt};
use twilight_http::Error;
use twilight_model::channel::message::AllowedMentions;
use twilight_model::gateway::payload::incoming::PresenceUpdate;
use twilight_model::gateway::presence::{ActivityType, Status};
use twilight_model::id::marker::{ChannelMarker, RoleMarker};
use twilight_model::id::Id;
use vesper::prelude::*;

pub const MEMBER_ROLE_ID: Id<RoleMarker> = Id::<RoleMarker>::new(1201246584382439475u64);
pub const PROMOTER_ROLE_ID: Id<RoleMarker> = Id::<RoleMarker>::new(1297596217198510210u64);
pub const LOG_CHANNEL_ID: Id<ChannelMarker> = Id::<ChannelMarker>::new(1299148030104305674u64);

pub async fn handle_shard_events(shard: &mut Shard, framework: Arc<Framework<()>>) {
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
                    .add_guild_member_role(member.guild_id, member.user.id, MEMBER_ROLE_ID)
                    .await
                    .unwrap();
            }
            Event::PresenceUpdate(presence) => {
                handle_presence_update(&framework, presence).await;
            }
            _ => (),
        }
    }
}

async fn handle_presence_update(framework: &Framework<()>, presence: Box<PresenceUpdate>) {
    if presence.status == Status::Offline {
        return;
    };

    let custom_activity = presence.activities.iter().any(|a| {
        a.kind == ActivityType::Custom
            && a.state
                .as_ref()
                .map_or(false, |state| state.contains("dbots.fun"))
    });

    sleep(std::time::Duration::from_secs(5)).await;

    let user_id = presence.user.id();

    let has_role = framework
        .http_client()
        .guild_member(presence.guild_id, user_id)
        .await
        .unwrap()
        .model()
        .await
        .unwrap()
        .roles
        .contains(&PROMOTER_ROLE_ID);

    if custom_activity && !has_role {
        framework
            .http_client()
            .add_guild_member_role(presence.guild_id, presence.user.id(), PROMOTER_ROLE_ID)
            .await
            .unwrap();

        send_log_message(
            framework.http_client(),
            &format!("`+` Added promoter role to user: <@{user_id}> (`{user_id}`)"),
        )
        .await
        .unwrap();
    } else if !custom_activity && has_role {
        framework
            .http_client()
            .remove_guild_member_role(presence.guild_id, presence.user.id(), PROMOTER_ROLE_ID)
            .await
            .unwrap();

        send_log_message(
            framework.http_client(),
            &format!("`-` Removed promoter role from user: <@{user_id}> (`{user_id}`)"),
        )
        .await
        .unwrap();
    }
}

async fn send_log_message(http_client: &twilight_http::Client, message: &str) -> Result<(), Error> {
    http_client
        .create_message(LOG_CHANNEL_ID)
        .content(message)
        .allowed_mentions(Some(&AllowedMentions {
            parse: vec![],
            replied_user: false,
            roles: vec![],
            users: vec![],
        }))
        .await?;

    Ok(())
}
