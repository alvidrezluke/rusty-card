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
        pub rarity: String,
        pub id: String,
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
        let rolled_id = rm_quotes(v["documents"][index]["fields"]["id"]["stringValue"].to_string());
        let genCard = GeneratedCard {
            name: rolled_name,
            image: rolled_image,
            category: rolled_category,
            subcategory: rolled_subcategory,
            rarity: rolled_rarity,
            id: rolled_id
        };
        Ok(genCard)
    }

    async fn create_user(id: String, card_id: String) -> Result<()> {
        let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/users?documentId={user_id}", project_id = get_project_id(), user_id = id);

        let mut card = HashMap::new();
        card.insert("stringValue", card_id);
        let values = [card];

        let mut array_value = HashMap::new();
        array_value.insert("values", values);
        let mut cards = HashMap::new();
        cards.insert("arrayValue", array_value);
        let mut fields = HashMap::new();
        fields.insert("cards", cards);
        let mut map = HashMap::new();
        map.insert("fields", fields);
        

        let client = reqwest::Client::new();
        let response = client.post(request_url)
            .json(&map)
            .send()
            .await;
        
        let status = response.expect("Uh oh.").text().await.expect("Uh oh. 1");
        Ok(())
    }

    async fn user_exists(id: String, card_id: String) -> bool {
        let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/users/{user_id}", project_id = get_project_id(), user_id = id);

        let response = reqwest::get(request_url).await.unwrap();
        let status = response.text().await.expect("Uh oh. 1");
        let v: Value = serde_json::from_str(status.as_str()).expect("Uh oh. 2");
        if v["error"]["code"] == 404 {
            create_user(id, card_id).await;
        } else {
            add_card(id, card_id).await;
        }
        true
    }

    async fn add_card(user_id: String, card_id: String) -> Result<()> {
        let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/users/{user_id}", project_id = get_project_id(), user_id = user_id);
        let response = reqwest::get(&request_url).await.unwrap();
        let status = response.text().await.expect("Uh oh. 1");
        let v: Value = serde_json::from_str(status.as_str()).expect("Uh oh. 2");
        let card = json!({
            "stringValue": card_id
        });
        let mut newCards = v["fields"]["cards"]["arrayValue"]["values"].as_array().expect("Uh oh.").clone();
        newCards.push(card);

        let mut array_value = HashMap::new();
        array_value.insert("values", newCards);
        let mut cards = HashMap::new();
        cards.insert("arrayValue", array_value);
        let mut fields = HashMap::new();
        fields.insert("cards", cards);
        let mut map = HashMap::new();
        map.insert("fields", fields);


        let client = reqwest::Client::new();
        let response = client.patch(&request_url)
            .json(&map)
            .send()
            .await;
        
        let status = response.expect("Uh oh.").text().await.expect("Uh oh. 1");
        Ok(())
    }

    pub async fn save_card(user_id: String, card_id: String) -> Result<()> {
        user_exists(user_id, card_id).await;
        Ok(())
    }

    async fn get_card(card_id: String) -> Result<GeneratedCard> {
        let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/cards/{card_id}", project_id = get_project_id(), card_id = card_id);
        let response = reqwest::get(request_url).await.unwrap();
        let text = response.text().await.unwrap();
        let v: Value = serde_json::from_str(text.as_str())?;
        let rolled_name = rm_quotes(v["fields"]["name"]["stringValue"].to_string());
        let rolled_image = rm_quotes(v["fields"]["image"]["stringValue"].to_string());
        let rolled_category = rm_quotes(v["fields"]["category"]["stringValue"].to_string());
        let rolled_subcategory = rm_quotes(v["fields"]["subcategory"]["stringValue"].to_string());
        let rolled_rarity = rm_quotes(v["fields"]["rarity"]["stringValue"].to_string());
        let rolled_id = rm_quotes(v["fields"]["id"]["stringValue"].to_string());
        let genCard = GeneratedCard {
            name: rolled_name,
            image: rolled_image,
            category: rolled_category,
            subcategory: rolled_subcategory,
            rarity: rolled_rarity,
            id: rolled_id
        };
        Ok(genCard)
    }

    pub async fn fetch_inventory(user_id: String) -> Vec<GeneratedCard> {
        let request_url = format!("https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/users/{user_id}", project_id = get_project_id(), user_id = user_id);
        let response = reqwest::get(&request_url).await.unwrap();
        let status = response.text().await.expect("Uh oh. 1");
        let v: Value = serde_json::from_str(status.as_str()).expect("Uh oh. 2");
        let owned_cards = v["fields"]["cards"]["arrayValue"]["values"].as_array().unwrap().to_vec();
        let mut display_vec = vec![];
        for card in owned_cards {
            let card_id = rm_quotes(card["stringValue"].to_string());
            display_vec.push(get_card(card_id).await.expect("Card does not exist"));
        }
        display_vec
    }