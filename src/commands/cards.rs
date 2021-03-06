use std::time::Duration;

use serenity::framework::standard::macros::command;
use serenity::framework::standard::{CommandResult, Args};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::firebase;
use crate::interactions;
use crate::misc;
use crate::config;

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
    //  Parse args
    let passed_args = args.rest().to_string();
    let mut split_args = passed_args.split_whitespace();
    let category_option = split_args.next();
    if category_option.is_none() {
        msg.reply(ctx, "You must supply a category when you use this function. Examples: c, characters, p, posters").await?;
        return Ok(());
    }

    //  Parse category from string
    let category_result = misc::get_category(category_option.unwrap().to_string().to_lowercase());
    let category: String = match category_result {
        Ok(s) => {
            s
        },
        Err(e) => {
            interactions::send_error(ctx, msg, e).await?;
            return Ok(());
        }
    };

    // Check for duration
    let checked_time = firebase::check_roll_time(msg.author.id.to_string()).await.expect("Failed to get last rolled time.");
    if !checked_time {
        msg.reply(ctx, "You can only roll once every 15 minutes!").await?;
        return Ok(());
    }

    //  Get cards of that category
    let generated_card = firebase::get_cards(category).await;

    //  Send the rolled card to the user
    match generated_card {
        Ok(card) => {
            let _saved_status = firebase::save_card(msg.author.id.to_string(), card.id.clone()).await;
            if card.link.is_empty() {
                if let Err(why) = msg.channel_id.send_message(&ctx.http, |m| {
                    m.content(format!("{} rolled:", msg.author.mention())).embed(|e| e.title(card.name).description(card.set).footer(|f| f.text(format!("{} - ID: {}", card.theme, card.id))).image(card.image))
                }).await {
                    println!("Error sending message: {:?}", why);
                }
            } else {
                msg.channel_id.send_message(&ctx.http, |m| {
                    m.content(format!("{} rolled:", msg.author.mention())).embed(|e| e.title(card.name).url(card.link).description(card.set).footer(|f| f.text(format!("{} - ID: {}", card.theme, card.id))).image(card.image))
                }).await?;
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
    let character_category_alternate: Vec<String> = vec![
        "characters".to_string(),
        "character".to_string(),
        "chars".to_string(),
        "char".to_string(),
        "c".to_string(),
    ];

    let posters_category_alternate: Vec<String> = vec![
        "posters".to_string(),
        "poster".to_string(),
        "post".to_string(),
        "p".to_string(),
    ];
    let passed_args = args.rest().to_string();
    let mut split_args = passed_args.split_whitespace();
    let category_option = split_args.next();
    if category_option.is_none() {
        msg.reply(ctx, "You must supply a category when you use this function. Examples: c, characters, p, posters").await?;
        return Ok(());
    }
    let mut category = category_option.unwrap().to_string().to_lowercase();
    if character_category_alternate.contains(&category) {
        category = "characters".to_string();
    } else if posters_category_alternate.contains(&category) {
        category = "posters".to_string();
    }
    if !(category == "characters" || category == "posters") {
        interactions::send_error(ctx, msg, format!("Did not recognize category: {}. Valid categories include \"characters\" and \"posters\".", category)).await?;
        return Ok(());
    }

    let inventory_status = firebase::check_inventory_time(msg.author.id.to_string()).await;
    match inventory_status {
        Ok(b) => {
            if !b {
                msg.reply(ctx, format!("Can only run the inventory command every {} minutes.", config::INVTIME)).await?;
                return Ok(());
            }
        },
        Err(e) => {
            msg.reply(ctx, format!("Error: {}", e)).await?;
            return Ok(());
        }
    }


    let inventory = firebase::fetch_inventory(msg.author.id.to_string(), category).await;
    if inventory.is_empty() {
        msg.reply(ctx, "You do not have any cards! Roll for them using !r (category).").await?;
        return Ok(());
    }
    let mut card_index = 0;
    let mut timer = Duration::from_secs(300);
    let length = inventory.len() - 1;
    let emojis = [
        ReactionType::from('???'),
        ReactionType::from('???'),
    ];
    let forward_emoji = [
        ReactionType::from('???'),
    ];
    let backward_emoji = [
        ReactionType::from('???'),
    ];
    let mut message = msg.channel_id.send_message(&ctx.http, |m| {
        if inventory[card_index].link.is_empty() {
            m.content(format!("{}'s inventory: Card {}/{}", msg.author.mention(), 1, length + 1)).embed(|e| e.title(&inventory[0].name).description(&inventory[0].set).footer(|f| f.text(format!("{} - ID: {} - Quantity: {}", &inventory[0].theme, &inventory[0].id, &inventory[0].quantity))).image(&inventory[0].image))
        } else {
            m.content(format!("{}'s inventory: Card {}/{}", msg.author.mention(), 1, length + 1)).embed(|e| e.title(&inventory[0].name).url(&inventory[0].link).description(&inventory[0].set).footer(|f| f.text(format!("{} - ID: {} - Quantity: {}", &inventory[0].theme, &inventory[0].id, &inventory[0].quantity))).image(&inventory[0].image))
        }
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
                if inventory[card_index].link.is_empty() {
                    m.content(format!("{}'s inventory: Card {}/{}", msg.author.mention(), card_index + 1, length + 1)).embed(|e| e.title(&inventory[card_index].name).description(&inventory[card_index].set).footer(|f| f.text(format!("{} - ID: {} - Quantity: {}", &inventory[card_index].theme, &inventory[card_index].id, &inventory[card_index].quantity))).image(&inventory[card_index].image))
                } else {
                    m.content(format!("{}'s inventory: Card {}/{}", msg.author.mention(), card_index + 1, length + 1)).embed(|e| e.title(&inventory[card_index].name).url(&inventory[card_index].link).description(&inventory[card_index].set).footer(|f| f.text(format!("{} - ID: {} - Quantity: {}", &inventory[card_index].theme, &inventory[card_index].id, &inventory[card_index].quantity))).image(&inventory[card_index].image))
                }
            }).await?;
            message.delete_reactions(ctx).await?;
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
                if inventory[card_index].link.is_empty() {
                    m.content(format!("{}'s inventory: Card {}/{}", msg.author.mention(), card_index + 1, length + 1)).embed(|e| e.title(&inventory[card_index].name).description(&inventory[card_index].set).footer(|f| f.text(format!("{} - ID: {} - Quantity: {}", &inventory[card_index].theme, &inventory[card_index].id, &inventory[card_index].quantity))).image(&inventory[card_index].image))
                } else {
                    m.content(format!("{}'s inventory: Card {}/{}", msg.author.mention(), card_index + 1, length + 1)).embed(|e| e.title(&inventory[card_index].name).url(&inventory[card_index].link).description(&inventory[card_index].set).footer(|f| f.text(format!("{} - ID: {} - Quantity: {}", &inventory[card_index].theme, &inventory[card_index].id, &inventory[card_index].quantity))).image(&inventory[card_index].image))
                }
            }).await?;
            message.delete_reactions(ctx).await?;
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
    let status = firebase::trade_card(msg.author.id.to_string(), card_id.to_string(), user_id.to_string()).await.clone();
    if status.is_err() {
        msg.reply(&ctx, format!("Error: {}", status.err().expect("Invalid Error"))).await?;
        return Ok(());
    }
    msg.reply(&ctx, format!("Successfully transferred card: {}.", card_id)).await?;
    Ok(())
}

#[command]
#[aliases("h")]
pub async fn help (ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx, "To roll a card use the command \"!roll (category)\". The current category options are characters or posters. You can view your inventory with \"!inventory (category)\"").await?;
    Ok(())
}