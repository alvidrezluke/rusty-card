use crate::{error::Error};

use serenity::{
    collector::ReactionAction,
    futures::StreamExt,
    model::prelude::{Message, ReactionType, User},
    prelude::Context,
};
use std::time::Duration;

pub async fn add_reactions(
    ctx: &Context,
    msg: &Message,
    emojis: Vec<ReactionType>,
) -> Result<(), Error> {
    let channel_id = msg.channel_id;
    let msg_id = msg.id;
    let http = ctx.http.clone();

    tokio::spawn(async move {
        for emoji in emojis {
            http.create_reaction(channel_id.0, msg_id.0, &emoji).await?;
        }

        Result::<_, Error>::Ok(())
    });

    Ok(())
}

pub async fn reaction_prompt(
    ctx: &Context,
    msg: &Message,
    user: &User,
    emojis: &[ReactionType],
    timeout: f32,
) -> Result<(usize, ReactionType), Error> {
    add_reactions(ctx, msg, emojis.to_vec()).await?;

    let mut collector = user
        .await_reactions(&ctx)
        .message_id(msg.id)
        .timeout(Duration::from_secs_f32(timeout)).build();

    while let Some(action) = collector.next().await {
        if let ReactionAction::Added(reaction) = action.as_ref() {
            if emojis.contains(&reaction.emoji) {
                return Ok((
                    emojis.iter().position(|p| p == &reaction.emoji).unwrap(),
                    reaction.emoji.clone(),
                ));
            }
        }
    }

    Err(Error::TimeoutError)
}