    use serde::Deserialize;
    use serde_json::{Result, Value, json};
    use std::{env, collections::HashMap, thread, time};
    use rand::Rng;
    use std::time::Duration;

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
        pub theme: String,
        pub id: String,
        pub quantity: u16,
    }

    pub fn rm_quotes(value: String) -> String {
        let mut chars = value.chars();
        chars.next();
        chars.next_back();
        chars.as_str().to_string()
    }
    
    pub async fn get_cards(category: String) -> Result<GeneratedCard> {
        let request_url = format!("https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/cards/{}/cards", get_project_id(), category);

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
        let rolled_theme = rm_quotes(v["documents"][index]["fields"]["theme"]["stringValue"].to_string());
        let rolled_id = rm_quotes(v["documents"][index]["fields"]["id"]["stringValue"].to_string());
        let genCard = GeneratedCard {
            name: rolled_name,
            image: rolled_image,
            category: rolled_category,
            subcategory: rolled_subcategory,
            theme: rolled_theme,
            id: rolled_id,
            quantity: 1,
        };
        Ok(genCard)
    }

    

    // async fn user_exists(id: String, card_id: String) -> bool {
    //     let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/users/{user_id}", project_id = get_project_id(), user_id = id);

    //     let response = reqwest::get(request_url).await.unwrap();
    //     let status = response.text().await.expect("Uh oh. 1");
    //     let v: Value = serde_json::from_str(status.as_str()).expect("Uh oh. 2");
    //     if v["error"]["code"] == 404 {
    //         create_user(id, card_id).await;
    //     } else {
    //         add_card(id, card_id).await;
    //     }
    //     true
    // }

    // async fn add_card(user_id: String, card_id: String) -> Result<()> {
    //     let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/users/{user_id}", project_id = get_project_id(), user_id = user_id);
    //     let response = reqwest::get(&request_url).await.unwrap();
    //     let status = response.text().await.expect("Uh oh. 1");
    //     let v: Value = serde_json::from_str(status.as_str()).expect("Uh oh. 2");
    //     let card = json!({
    //         "stringValue": card_id
    //     });
    //     let mut newCards = v["fields"]["cards"]["arrayValue"]["values"].as_array().expect("Uh oh.").clone();
    //     newCards.push(card);

    //     let mut array_value = HashMap::new();
    //     array_value.insert("values", newCards);
    //     let mut cards = HashMap::new();
    //     cards.insert("arrayValue", array_value);
    //     let mut fields = HashMap::new();
    //     fields.insert("cards", cards);
    //     let mut map = HashMap::new();
    //     map.insert("fields", fields);


    //     let client = reqwest::Client::new();
    //     let response = client.patch(&request_url)
    //         .json(&map)
    //         .send()
    //         .await;
        
    //     let status = response.expect("Uh oh.").text().await.expect("Uh oh. 1");
    //     Ok(())
    // }

    // pub async fn save_card(user_id: String, card_id: String) -> Result<()> {
    //     user_exists(user_id, card_id).await;
    //     Ok(())
    // }

    async fn get_card(card_id: String, quantity: u16, category: String) -> Result<GeneratedCard> {
        let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/cards/{category}/cards/{card_id}", project_id = get_project_id(), category = category, card_id = card_id);
        let response = reqwest::get(request_url).await.unwrap();
        let text = response.text().await.unwrap();
        let v: Value = serde_json::from_str(text.as_str())?;
        let rolled_name = rm_quotes(v["fields"]["name"]["stringValue"].to_string());
        let rolled_image = rm_quotes(v["fields"]["image"]["stringValue"].to_string());
        let rolled_category = rm_quotes(v["fields"]["category"]["stringValue"].to_string());
        let rolled_subcategory = rm_quotes(v["fields"]["subcategory"]["stringValue"].to_string());
        let rolled_theme = rm_quotes(v["fields"]["theme"]["stringValue"].to_string());
        let rolled_id = rm_quotes(v["fields"]["id"]["stringValue"].to_string());
        let genCard = GeneratedCard {
            name: rolled_name,
            image: rolled_image,
            category: rolled_category,
            subcategory: rolled_subcategory,
            theme: rolled_theme,
            id: rolled_id,
            quantity: quantity
        };
        Ok(genCard)
    }

    pub async fn fetch_inventory(user_id: String, category: String) -> Vec<GeneratedCard> {
        let owned_cards = get_user_cards(user_id).await.expect("No cards found");
        if owned_cards.len() == 0 {
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
        // name: String,
        // category: String,
        // subcategory: String,
        // theme: String,
        // image: String
    }

    async fn get_user_cards(user_id: String) -> Result<Vec<CollectionCard>> {
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

    async fn create_user(id: String, json_value: Value) -> Result<()> {
        let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/users?documentId={user_id}", project_id = get_project_id(), user_id = id);

        let client = reqwest::Client::new();
        let response = client.post(request_url)
            .json(&json_value)
            .send()
            .await;
        
        let status = response.expect("Uh oh.").text().await.expect("Uh oh. 1");
        Ok(())
    }

    pub async fn save_card(user_id: String, card_id: String) -> Result<()> {
        let cards: Vec<CollectionCard> = get_user_cards(user_id.clone()).await?;
        if cards.len() == 0 {
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

        let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/users/{user_id}", project_id = get_project_id(), user_id = user_id);

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
        
        let status = response.expect("Uh oh.").text().await.expect("Uh oh. 1");

        Ok(())
    }

    pub async fn trade_card(from_user_id: String, card_id: String, to_user_id: String) -> Result<()> {
        let collection = get_user_cards(from_user_id.clone()).await?;
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
        if !found {
            println!("User does not have this card");
            return Ok(());
        }

        if short_collection.to_vec().is_empty() {
            let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/users/{user_id}", project_id = get_project_id(), user_id = from_user_id);
            let client = reqwest::Client::new();
            let response = client.delete(&request_url).send().await;
            let status = response.expect("Failed to delete user");
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
            
            let status = response.expect("Uh oh.").text().await.expect("Uh oh. 1");
        }
        
        save_card(to_user_id, card_id).await;
        Ok(())
    }