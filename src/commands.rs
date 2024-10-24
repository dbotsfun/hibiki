use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::marker::RoleMarker;
use twilight_model::id::Id;
use vesper::prelude::*;

#[command]
#[description("Check how many promoters there are")]
pub async fn promoters(ctx: &SlashContext<()>) -> DefaultCommandResult {
    let members = ctx
        .http_client()
        .guild_members(ctx.interaction.guild_id.unwrap())
        .await?
        .model()
        .await?;
    let promoters = members
        .iter()
        .filter(|member| {
            member
                .roles
                .contains(&Id::<RoleMarker>::new(1297596217198510210u64))
        })
        .count();

    ctx.interaction_client
        .create_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some(format!("There are {promoters} promoters right now!")),
                    ..Default::default()
                }),
            },
        )
        .await?;

    Ok(())
}
