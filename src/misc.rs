use crate::config;

pub fn get_category(input: String) -> Result<String, String> {
    if config::character_category_alternates.contains(&input.as_str()) {
        Ok("characters".to_string())
    } else if config::posters_category_alternates.contains(&input.as_str()) {
        Ok("posters".to_string())
    } else {
        return Err(format!("Did not recognize category: {}. Valid categories include \"characters\" and \"posters\".", input));
    }
}