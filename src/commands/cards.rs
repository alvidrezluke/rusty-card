use std::time::Duration;

use serenity::framework::standard::macros::command;
use serenity::framework::standard::{CommandResult, Args};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::firebase;
use crate::firebase::GeneratedCard;
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
pub async fn roll(ctx: &Context, msg: &Message) -> CommandResult {
    let generatedCard = firebase::get_cards().await;
    match generatedCard {
        Ok(card) => {
            firebase::save_card(msg.author.id.to_string(), card.id).await;
            if let Err(why) = msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| e.title(card.name).description(format!("Rarity: {}\nCategory: {}\nSubcategory: {}", card.rarity, card.category, card.subcategory)).image(card.image))
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
async fn r(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    roll(ctx, msg, args).await
}

#[command]
pub async fn inventory(ctx: &Context, msg: &Message) -> CommandResult {
    let inventory = firebase::fetch_inventory(msg.author.id.to_string()).await;
    let mut card_index = 0;
    let mut timer = Duration::from_secs(300);
    let length = inventory.len() - 1;
    let emojis = [
        ReactionType::from('⬅'),
        ReactionType::from('➡'),
    ];
    let mut message = msg.channel_id.send_message(&ctx.http, |m| {
        m.content(format!("{}/{}", card_index + 1, length + 1)).embed(|e| e.title(&inventory[0].name).description(format!("Rarity: {}\nCategory: {}\nSubcategory: {}\nID: {}", inventory[0].rarity, inventory[0].category, inventory[0].subcategory), inventory[0].id).image(&inventory[0].image))
    }).await.expect("Failed to send message");

    let mut selection = interactions::reaction_prompt(ctx, &message, &msg.author, &emojis, 30.0).await?;

    while !timer.is_zero() {
        let (index, _) = selection;
        if index == 0 && card_index > 0 {
            card_index -= 1;
            message.edit(&ctx.http, |m| {
                m.content(format!("{}/{}", card_index + 1, length + 1)).embed(|e| e.title(&inventory[card_index].name).description(format!("Rarity: {}\nCategory: {}\nSubcategory: {}\nID: {}", &inventory[card_index].rarity, &inventory[card_index].category, &inventory[card_index].subcategory, &inventory[card_index].id)).image(&inventory[card_index].image))
            }).await;
            message.delete_reactions(ctx).await;
            selection = interactions::reaction_prompt(ctx, &message, &msg.author, &emojis, 30.0).await?;
            timer = Duration::from_secs(300);
        } else if index == 1 && card_index < length {
            card_index += 1;
            message.edit(&ctx.http, |m| {
                m.content(format!("{}/{}", card_index + 1, length + 1)).embed(|e| e.title(&inventory[card_index].name).description(format!("Rarity: {}\nCategory: {}\nSubcategory: {}\nID: {}", &inventory[card_index].rarity, &inventory[card_index].category, &inventory[card_index].subcategory, &inventory[card_index].id)).image(&inventory[card_index].image))
            }).await;
            message.delete_reactions(ctx).await;
            selection = interactions::reaction_prompt(ctx, &message, &msg.author, &emojis, 30.0).await?;
            timer = Duration::from_secs(300);
        }
    }   
    Ok(())
}

#[command]
async fn i(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    inventory(ctx, msg, args).await
}

