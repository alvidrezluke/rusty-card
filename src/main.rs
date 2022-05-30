use std::env;
use std::time::Instant;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use dotenv::dotenv;
use crate::firebase::get_cards;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let timer = Instant::now();
        if msg.content.chars().take(1).collect::<String>() == "!" && !msg.author.bot {
            match msg.content.as_str() {
                "!ping" => {
                    if let Err(why) = msg.channel_id.say(&ctx.http, format!("Pong!\nThis took {} nanoseconds from the message being recieved to be processed.", timer.elapsed().as_nanos())).await {
                        println!("Error sending message: {:?}", why);
                    }
                },
                "!rick" => {
                    if let Err(why) = msg.channel_id.send_message(&ctx.http, |m| {
                        m.embed(|e| e.title("Here is an image").description("With a description").image("https://firebasestorage.googleapis.com/v0/b/rusty-cards.appspot.com/o/rickroll-roll.gif?alt=media&token=269ee9e8-eba9-4cc3-8caf-93f554e07c4c"))
                    }).await {
                        println!("Error sending message: {:?}", why);
                    }
                },
                "!roll" => {
                    let generatedCard = get_cards().await;
                    match generatedCard {
                        Ok(card) => {
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
                _ => println!("Command not recognized"),
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

mod firebase {
    use reqwest::Error;
    use serde::Deserialize;
    use serde_json::{Result, Value};
    use std::{env, collections::HashMap};
    use rand::Rng;
    const URL: &'static str = "https://firestore.googleapis.com";
    const DOMAIN: &'static str = "firestore.googleapis.com";

    pub type BoxError = Box<dyn std::error::Error + Sync + Send + 'static>;
    fn get_token() -> String {
        println!("{}", env::var("FIREBASE_TOKEN").unwrap());
        env::var("FIREBASE_TOKEN").unwrap()
    }

    fn get_project_id() -> String {
        env::var("PROJECT_ID").unwrap()
    }

    #[derive(Deserialize, Debug)]
    pub struct GeneratedCard {
        pub name: String,
        pub image: String,
        pub category: String,
        pub subcategory: String,
        pub rarity: String,
    }

    fn rm_quotes(value: String) -> String {
        let mut chars = value.chars();
        chars.next();
        chars.next_back();
        chars.as_str().to_string()
    }
    
    pub async fn get_cards() -> Result<GeneratedCard> {
        let request_url = format!("https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/cards", project_id = get_project_id());

        let response = reqwest::get(request_url).await.unwrap();
        let text = response.text().await.unwrap();
        let v: Value = serde_json::from_str(text.as_str())?;
        let length = v["documents"].as_array().expect("Uh oh.").len();
        let mut rng = rand::thread_rng();
        let index: usize = rng.gen_range(0..length);
        let rolled_name = rm_quotes(v["documents"][index]["fields"]["name"]["stringValue"].to_string());
        let rolled_image = rm_quotes(v["documents"][index]["fields"]["image"]["stringValue"].to_string());
        let rolled_category = rm_quotes(v["documents"][index]["fields"]["category"]["stringValue"].to_string());
        let rolled_subcategory = rm_quotes(v["documents"][index]["fields"]["subcategory"]["stringValue"].to_string());
        let rolled_rarity = rm_quotes(v["documents"][index]["fields"]["rarity"]["stringValue"].to_string());
        println!("You rolled:\nName: {}\nImage: {}", rolled_name, rolled_image);
        let genCard = GeneratedCard {
            name: rolled_name,
            image: rolled_image,
            category: rolled_category,
            subcategory: rolled_subcategory,
            rarity: rolled_rarity,
        };
        Ok(genCard)
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