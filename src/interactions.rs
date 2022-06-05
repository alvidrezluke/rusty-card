use serenity::{
    collector::ReactionAction,
    futures::StreamExt,
    model::prelude::{Message, ReactionType, User},
    prelude::Context,
};
use std::time::Duration;

use serenity::Error as SerenityError;
use std::{
    borrow::Cow,
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};

/// A common error type for all functions and methods of the library.
///
/// It can be directly converted into serenity's [`Error`](SerenityError).
#[derive(Debug)]
pub enum Error {
    /// Error returned by serenity.
    SerenityError(SerenityError),
    /// Error returned when an operation times out.
    TimeoutError,
    /// Error returned when user's choice is invalid.
    InvalidChoice,
    /// Error returned for all other cases.
    Other(String),
}

impl StdError for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let err = match self {
            Error::SerenityError(e) => Cow::from(e.to_string()),
            Error::TimeoutError => Cow::from("You took too long to respond."),
            Error::InvalidChoice => Cow::from("Invalid choice!"),
            Error::Other(e) => Cow::from(e),
        };

        write!(f, "{}", err)
    }
}

impl<'a> From<&'a str> for Error {
    fn from(error: &'a str) -> Self {
        Self::Other(error.to_string())
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Self::Other(error)
    }
}

impl From<SerenityError> for Error {
    fn from(error: SerenityError) -> Self {
        Self::SerenityError(error)
    }
}

pub async fn add_reactions(
    ctx: &Context,
    msg: &Message,
    emojis: Vec<ReactionType>,
) -> Result<(), String> {
    let channel_id = msg.channel_id;
    let msg_id = msg.id;
    let http = ctx.http.clone();

    tokio::spawn(async move {
        for emoji in emojis {
            http.create_reaction(channel_id.0, msg_id.0, &emoji).await?;
        }

        Result::<(), Error>::Ok(())
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

pub async fn send_error(ctx: &Context, msg: &Message, error_message: String) -> Result<(), String> {
    let status = msg.channel_id.say(ctx.clone().http, error_message).await;
    match status {
        Ok(_) => {
            Ok(())
        },
        Err(e) => {
            println!("Error: {}", e);
            Ok(())
        }
    }
}