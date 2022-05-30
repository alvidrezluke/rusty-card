use std::env;
use std::time::Instant;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::model::prelude::ReactionType;
use crate::interactions::reaction_prompt;

use dotenv::dotenv;

mod firebase;
mod interactions;
mod error;

#[doc(inline)]
pub use error::Error;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let timer = Instant::now();
        if msg.content.chars().take(1).collect::<String>() == "!" && !msg.author.bot {
            match msg.content.as_str() {
                "!rick" => {
                    if let Err(why) = msg.channel_id.send_message(&ctx.http, |m| {
                        m.embed(|e| e.title("Here is an image").description("With a description").image("https://firebasestorage.googleapis.com/v0/b/rusty-cards.appspot.com/o/rickroll-roll.gif?alt=media&token=269ee9e8-eba9-4cc3-8caf-93f554e07c4c"))
                    }).await {
                        println!("Error sending message: {:?}", why);
                    }
                },
                "!roll" => {
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
                    
                },
                "!inventory" => {
                    let inventory = firebase::fetch_inventory(msg.author.id.to_string()).await;
                    for card in inventory {
                        println!("{:?}", card);

                        let message = msg.channel_id.send_message(&ctx.http, |m| {
                            m.embed(|e| e.title(card.name).description(format!("Rarity: {}\nCategory: {}\nSubcategory: {}", card.rarity, card.category, card.subcategory)).image(card.image))
                        }).await.expect("Failed to send message");

                        let emojis = [
                            ReactionType::from('⬅'),
                            ReactionType::from('➡'),
                        ];

                        let (index, _emoji) = reaction_prompt(ctx, &prompt_msg, &msg.author, &emojis, 30.0).await?;
                    
                        if index == 0 {
                            // The user reacted with `🐶`.
                            msg.reply(&ctx.http, format!("I like {} more!", emojis[1]))
                                .await?;
                        } else {
                            // The user reacted with `🐱`.
                            msg.reply(&ctx.http, format!("I like {} more!", emojis[0]))
                                .await?;
                        }
                    }
                },
                _ => println!("Command not recognized"),
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let discord_token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&discord_token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}