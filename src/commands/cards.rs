use std::time::Duration;

use serenity::framework::standard::macros::command;
use serenity::framework::standard::{CommandResult, Args};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::firebase;
use crate::firebase::rm_quotes;
use crate::interactions;
use crate::error::Error;

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}

#[command]
pub async fn rick(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| e.image("https://firebasestorage.googleapis.com/v0/b/rusty-cards.appspot.com/o/rickroll-roll.gif?alt=media&token=269ee9e8-eba9-4cc3-8caf-93f554e07c4c"))
    }).await?;

    Ok(())
}

#[command]
#[aliases("r")]
pub async fn roll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let passed_args = args.rest().to_string();
    let mut split_args = passed_args.split_whitespace();
    let mut category = split_args.next().unwrap().to_string().to_lowercase();
    let generatedCard = firebase::get_cards(category).await;
    match generatedCard {
        Ok(card) => {
            firebase::save_card(msg.author.id.to_string(), card.id.clone()).await;
            if let Err(why) = msg.channel_id.send_message(&ctx.http, |m| {
                m.content(format!("{} rolled:", msg.author.mention())).embed(|e| e.title(card.name).description(format!("{}", card.set)).footer(|f| f.text(format!("{} - ID: {}", card.theme, card.id))).image(card.image))
            }).await {
                println!("Error sending message: {:?}", why);
            }
        },
        Err(e) => {
            if let Err(why) = msg.channel_id.say(&ctx.http, format!("Error: {}", e)).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }
    Ok(())
}

#[command]
#[aliases("i")]
pub async fn inventory(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let passed_args = args.rest().to_string();
    let mut split_args = passed_args.split_whitespace();
    let category = split_args.next().unwrap().to_string();
    let inventory = firebase::fetch_inventory(msg.author.id.to_string(), category).await;
    let mut card_index = 0;
    let mut timer = Duration::from_secs(300);
    let length = inventory.len() - 1;
    let emojis = [
        ReactionType::from('⬅'),
        ReactionType::from('➡'),
    ];
    let forward_emoji = [
        ReactionType::from('➡'),
    ];
    let backward_emoji = [
        ReactionType::from('⬅'),
    ];
    let mut message = msg.channel_id.send_message(&ctx.http, |m| {
        m.content(format!("{}'s inventory: Card {}/{}", msg.author.mention(), card_index + 1, length + 1)).embed(|e| e.title(&inventory[0].name).description(format!("{}", inventory[0].set)).footer(|f| f.text(format!("{} - ID: {} - Quantity: {}", inventory[0].theme, inventory[0].id, inventory[0].quantity))).image(&inventory[0].image))
    }).await.expect("Failed to send message");

    let mut selection = interactions::reaction_prompt(ctx, &message, &msg.author, &forward_emoji, 30.0).await?;

    let mut forward = true;
    let mut backward = false;

    while !timer.is_zero() {
        let (index, _) = selection;
        if (index == 0 && card_index > 0) || backward {
            backward = false;
            card_index -= 1;
            message.edit(&ctx.http, |m| {
                m.content(format!("{}'s inventory: Card {}/{}", msg.author.mention(), card_index + 1, length + 1)).embed(|e| e.title(&inventory[card_index].name).description(format!("{}", &inventory[card_index].set)).footer(|f| f.text(format!("{} - ID: {} - Quantity: {}", &inventory[card_index].theme, &inventory[card_index].id, &inventory[card_index].quantity))).image(&inventory[card_index].image))
            }).await;
            message.delete_reactions(ctx).await;
            if card_index == 0 {
                selection = interactions::reaction_prompt(ctx, &message, &msg.author, &forward_emoji, 30.0).await?;
                forward = true;
            } else {
                selection = interactions::reaction_prompt(ctx, &message, &msg.author, &emojis, 30.0).await?;
            }
            timer = Duration::from_secs(300);
        } else if (index == 1 && card_index < length) || forward {
            forward = false;
            card_index += 1;
            message.edit(&ctx.http, |m| {
                m.content(format!("{}'s inventory: Card {}/{}", msg.author.mention(), card_index + 1, length + 1)).embed(|e| e.title(&inventory[card_index].name).description(format!("{}", &inventory[card_index].set)).footer(|f| f.text(format!("{} - ID: {} - Quantity: {}", &inventory[card_index].theme, &inventory[card_index].id, &inventory[card_index].quantity))).image(&inventory[card_index].image))
            }).await;
            message.delete_reactions(ctx).await;
            if card_index == length {
                selection = interactions::reaction_prompt(ctx, &message, &msg.author, &backward_emoji, 30.0).await?;
                backward = true;
            } else {
                selection = interactions::reaction_prompt(ctx, &message, &msg.author, &emojis, 30.0).await?;
            }
            timer = Duration::from_secs(300);
        }
    }   
    Ok(())
}

#[command]
#[aliases("t")]
pub async fn trade(ctx: &Context, msg: &Message, args: Args) -> CommandResult {

    let passed_args = args.rest().to_string();
    let mut split_args = passed_args.split_whitespace();
    let mut user_id = split_args.next().unwrap();
    let user_id_len = user_id.len();
    user_id = &user_id[2..user_id_len-1];
    let card_id = split_args.next().unwrap();
    let status = firebase::trade_card(msg.author.id.to_string(), card_id.to_string(), user_id.to_string()).await?;
    msg.reply(&ctx, format!("Successfully transferred card: {}.", card_id)).await?;
    Ok(())
}