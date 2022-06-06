use serde::Deserialize;
use serde_json::{Value, json};
use std::env;
use rand::{Rng, prelude::SliceRandom, SeedableRng};
use chrono::{Utc, TimeZone, Duration};

use crate::config;

fn get_token() -> String {
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
    pub set: String,
    pub theme: String,
    pub id: String,
    pub quantity: u16,
    pub link: String
}

pub fn rm_quotes(value: String) -> String {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str().to_string()
}
    
pub async fn get_cards(category: String) -> Result<GeneratedCard, String> {
    let request_url = format!("https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/cards/{}/cards", get_project_id(), category);
    let response = reqwest::get(request_url).await.unwrap();
    let text = response.text().await.unwrap();
    let v: Value = serde_json::from_str(text.as_str()).expect("Failed to parse JSON.");
    let length = v["documents"].as_array().expect("Uh oh.").len();
    // let mut rng = rand::thread_rng();
    // let index: usize = rng.gen_range(0..length);
    // println!("{:?}", index);
    let array = v["documents"].as_array().expect("Not an array");
    let slice = array.choose(&mut rand::rngs::StdRng::from_entropy()).expect("Could not pick random number");
    let rolled_name = rm_quotes(slice["fields"]["name"]["stringValue"].to_string());
    let rolled_image = rm_quotes(slice["fields"]["image"]["stringValue"].to_string());
    let rolled_category = rm_quotes(slice["fields"]["category"]["stringValue"].to_string());
    let rolled_set = rm_quotes(slice["fields"]["set"]["stringValue"].to_string());
    let rolled_theme = rm_quotes(slice["fields"]["theme"]["stringValue"].to_string());
    let rolled_id = rm_quotes(slice["fields"]["id"]["stringValue"].to_string());
    let mut rolled_link = rm_quotes(slice["fields"]["link"]["stringValue"].to_string());
    if rolled_link == "ul" {
        rolled_link = String::new();
    }
    let gen_card = GeneratedCard {
        name: rolled_name,
        image: rolled_image,
        category: rolled_category,
        set: rolled_set,
        theme: rolled_theme,
        id: rolled_id,
        quantity: 1,
        link: rolled_link
    };
    Ok(gen_card)
}

pub async fn get_card(card_id: String, quantity: u16, category: String) -> Result<GeneratedCard, ()> {
    let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/cards/{category}/cards/{card_id}", project_id = get_project_id(), category = category, card_id = card_id);
    let response = reqwest::get(request_url).await.unwrap();
    let text = response.text().await.unwrap();
    let v: Value = serde_json::from_str(text.as_str()).expect("Failed to parse JSON from response.");
    let rolled_name = rm_quotes(v["fields"]["name"]["stringValue"].to_string());
    let rolled_image = rm_quotes(v["fields"]["image"]["stringValue"].to_string());
    let rolled_category = rm_quotes(v["fields"]["category"]["stringValue"].to_string());
    let rolled_set = rm_quotes(v["fields"]["set"]["stringValue"].to_string());
    let rolled_theme = rm_quotes(v["fields"]["theme"]["stringValue"].to_string());
    let rolled_id = rm_quotes(v["fields"]["id"]["stringValue"].to_string());
    let mut rolled_link = rm_quotes(v["fields"]["link"]["stringValue"].to_string());
    if rolled_link == "ul" {
        rolled_link = String::new();
    }
    let gen_card = GeneratedCard {
        name: rolled_name,
        image: rolled_image,
        category: rolled_category,
        set: rolled_set,
        theme: rolled_theme,
        id: rolled_id,
        quantity,
        link: rolled_link
    };
    Ok(gen_card)
}

pub async fn fetch_inventory(user_id: String, category: String) -> Vec<GeneratedCard> {
    let owned_cards = get_user_cards(user_id).await.expect("No cards found");
    if owned_cards.is_empty() {
        return vec![];
    }
    let mut display_vec = vec![];
    for card in owned_cards {
        let card_details = get_card(card.id, card.quantity, category.clone()).await.expect("Card does not exist");
        if card_details.category.to_lowercase() == category {
            display_vec.push(card_details);
        }
    }
    display_vec
}

struct CollectionCard {
    id: String,
    quantity: u16,
}

async fn get_user_cards(user_id: String) -> Result<Vec<CollectionCard>, ()> {
    let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/users/{user_id}", project_id = get_project_id(), user_id = user_id);
    let response = reqwest::get(&request_url).await.unwrap();
    if response.status().is_client_error() {
        return Ok(vec![]);
    }
    let status = response.text().await.expect("No cards found");
    
    let v: Value = serde_json::from_str(status.as_str()).expect("No valid JSON");
    let owned_cards = v["fields"]["cards"]["arrayValue"]["values"].as_array().unwrap().to_vec();
    let mut collection = vec![];

    for card in owned_cards {
        let card_id: String = rm_quotes(card["mapValue"]["fields"]["id"]["stringValue"].to_string());
        let card_quantity = rm_quotes(card["mapValue"]["fields"]["quantity"]["stringValue"].to_string()).parse::<u16>().unwrap();
        let collection_card = CollectionCard {
            id: card_id,
            quantity: card_quantity
        };
        collection.push(collection_card)
    }
    Ok(collection)
}

async fn create_user(id: String, json_value: Value) -> Result<(), ()> {
        let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/users?documentId={user_id}", project_id = get_project_id(), user_id = id);

        let client = reqwest::Client::new();
        let response = client.post(request_url)
            .json(&json_value)
            .send()
            .await;
        
        response.expect("Uh oh.").text().await.expect("Uh oh. 1");
        Ok(())
    }

pub async fn save_card(user_id: String, card_id: String) -> Result<(), ()> {
    let cards: Vec<CollectionCard> = get_user_cards(user_id.clone()).await?;
    if cards.is_empty() {
        let current_time = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let inv_time = (Utc::now() - Duration::minutes(6)).format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let json_data = json!({
            "fields": {
                "cards": {
                    "arrayValue": {
                        "values": [
                            {
                                "mapValue": {
                                    "fields": {
                                        "quantity": {
                                            "stringValue": "1"
                                        },
                                        "id": {
                                            "stringValue": card_id
                                        }
                                    }
                                }
                            }
                        ]
                    }
                },
                "last_rolled": {
                    "timestampValue": current_time
                },
                "last_inventory": {
                    "timestampValue": inv_time
                }
            }
        });
        create_user(user_id, json_data).await?;
        return Ok(());
    }
    let mut new_cards = vec![];
    let mut found = false;
    for mut card in cards {
        if card.id == card_id {
            card.quantity += 1;
            found = true;
        }
        let json_value = json!({
            "mapValue": {
                "fields": {
                    "quantity": {
                        "stringValue": card.quantity.to_string()
                    },
                    "id": {
                        "stringValue": card.id
                    }
                }
            }
        });
        new_cards.push(json_value);
    }
    if !found {
        let json_value = json!({
            "mapValue": {
                "fields": {
                    "quantity": {
                        "stringValue": "1"
                    },
                    "id": {
                        "stringValue": card_id
                    }
                }
            }
        });
        new_cards.push(json_value);
    }
    let request_url = format!("https://firestore.googleapis.com/v1beta1/projects/{project_id}/databases/(default)/documents/users/{user_id}?updateMask.fieldPaths=cards", project_id = get_project_id(), user_id = user_id);

    let patch_data = json!({
        "fields": {
            "cards": {
                "arrayValue": {
                    "values": new_cards.to_vec()
                }
            }
        }
    });


    let client = reqwest::Client::new();
    let response = client.patch(&request_url)
        .json(&patch_data)
        .send()
        .await;
        
    response.expect("Uh oh.").text().await.expect("Uh oh. 1");

    Ok(())
}

pub async fn trade_card(from_user_id: String, card_id: String, to_user_id: String) -> Result<(), String> {
    let collection = get_user_cards(from_user_id.clone()).await.expect("Failed to get user cards.");
    let mut short_collection = vec![];
    let mut found = false;
    for mut card in collection {
        if card.id == card_id {
            card.quantity -= 1;
            found = true;
        }
        if card.quantity != 0 {
            let json_value = json!({
                "mapValue": {
                    "fields": {
                        "quantity": {
                            "stringValue": card.quantity.to_string()
                        },
                        "id": {
                            "stringValue": card.id
                        }
                    }
                }
            });
            short_collection.push(json_value);
        }
    }
    if found {
        if short_collection.to_vec().is_empty() {
            let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/users/{user_id}", project_id = get_project_id(), user_id = from_user_id);
            let client = reqwest::Client::new();
            let response = client.delete(&request_url).send().await;
            response.expect("Failed to delete user");
        } else {
            let patch_data = json!({
                "fields": {
                    "cards": {
                        "arrayValue": {
                            "values": short_collection.to_vec()
                        }
                    }
                }
            });
            let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/users/{user_id}", project_id = get_project_id(), user_id = from_user_id);
            let client = reqwest::Client::new();
            let response = client.patch(&request_url)
                .json(&patch_data)
                .send()
                .await;
                
            response.expect("Uh oh.").text().await.expect("Uh oh. 1");
        }
        save_card(to_user_id, card_id).await.expect("Could not save card");
        Ok(())
    } else {
        Err("You do not have this card.".to_string())
    }
}

pub async fn check_roll_time(user_id: String) -> Result<bool, String> {
    let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/users/{user_id}", project_id = get_project_id(), user_id = user_id);
    let response = reqwest::get(request_url).await.unwrap();
    let text = response.text().await.unwrap();
    let v: Value = serde_json::from_str(text.as_str()).expect("Failed to parse JSON from response.");
    let raw_date = rm_quotes(v["fields"]["last_rolled"]["timestampValue"].to_string());
    if raw_date == "ul" {
        return Ok(true);
    }
    let last_rolled = Utc.datetime_from_str(raw_date.as_str(), "%Y-%m-%dT%H:%M:%SZ").expect("Invalid date").time();
    let current_time = Utc::now().time();
    let diff = (current_time - last_rolled).num_minutes();
    if diff > config::ROLLTIME {
        let update_status = update_roll_time(user_id).await;
        if update_status.is_err() {
            return Err("Could not update the status".to_string());
        }
        return Ok(true);
    }
    Ok(false)
}

async fn update_roll_time(user_id: String) -> Result<(), String> {
    let request_url = format!("https://firestore.googleapis.com/v1beta1/projects/{project_id}/databases/(default)/documents/users/{user_id}?updateMask.fieldPaths=last_rolled&alt=json", project_id = get_project_id(), user_id = user_id);
    let current_time = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let data = json!({
        "fields": {
            "last_rolled": {
                "timestampValue": current_time
            }
        }
    });

    let client = reqwest::Client::new();
    let response = client.patch(&request_url)
        .json(&data)
        .send()
        .await;
        
    response.expect("Uh oh.").text().await.expect("Uh oh. 1");
    Ok(())
}

pub async fn check_inventory_time(user_id: String) -> Result<bool, String> {
    let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/users/{user_id}", project_id = get_project_id(), user_id = user_id);
    let response = reqwest::get(request_url).await.unwrap();
    let text = response.text().await.unwrap();
    let v: Value = serde_json::from_str(text.as_str()).expect("Failed to parse JSON from response.");
    let raw_date = rm_quotes(v["fields"]["last_inventory"]["timestampValue"].to_string());
    if raw_date == "ul" {
        return Ok(true);
    }
    let last_rolled = Utc.datetime_from_str(raw_date.as_str(), "%Y-%m-%dT%H:%M:%SZ").expect("Invalid date").time();
    let current_time = Utc::now().time();
    let diff = (current_time - last_rolled).num_minutes();
    if diff > config::INVTIME {
        let update_status = update_inventory_time(user_id).await;
        if update_status.is_err() {
            return Err("Could not update the status".to_string());
        }
        return Ok(true);
    }
    Ok(false)
}

async fn update_inventory_time(user_id: String) -> Result<(), String> {
    let request_url = format!("https://firestore.googleapis.com/v1beta1/projects/{project_id}/databases/(default)/documents/users/{user_id}?updateMask.fieldPaths=last_inventory&alt=json", project_id = get_project_id(), user_id = user_id);
    let current_time = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let data = json!({
        "fields": {
            "last_inventory": {
                "timestampValue": current_time
            }
        }
    });

    let client = reqwest::Client::new();
    let response = client.patch(&request_url)
        .json(&data)
        .send()
        .await;
        
    response.expect("Uh oh.").text().await.expect("Uh oh. 1");
    Ok(())
}

