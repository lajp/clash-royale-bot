use reqwest::header::*;
use serde_json::Value;
use std::env;

#[derive(Debug)]
pub struct Chest {
    pub index: u32,
    pub name: String,
}

#[derive(Debug)]
pub struct Battle {
    pub time: i64,
    pub opponent: String,
    pub victory: bool,
    pub crowns: u8,
    pub opponent_crowns: u8,
}

pub async fn get_player_chests(id: &str) -> Result<Vec<Chest>, anyhow::Error> {
    let id = &id.replace('#', "%23");
    let token = env::var("CLASH_ROYALE_TOKEN").expect("Expected CLASH_ROYALE_TOKEN token");
    let token = format!("Bearer {}", token);
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, token.parse()?);
    let url = format!("https://api.clashroyale.com/v1/players/{}/upcomingchests", id);
    let client = reqwest::Client::new();
    let response = client.get(&url).headers(headers).send()
        .await?.text().await?;
    let response_json: Value = serde_json::from_str(&response)?;
    let mut ret = Vec::new();
    if let Some(chests) = response_json["items"].as_array() {
        for chest in chests {
            let index = chest["index"].as_i64().ok_or(anyhow::anyhow!("Unknown API error"))? as u32;
            let name = chest["name"].as_str().ok_or(anyhow::anyhow!("Unknown API error"))?.to_string();
            ret.push(Chest { index, name });
        }
    }
    Ok(ret)
}

pub async fn get_player_battles(id: &str) -> Result<Vec<Battle>, anyhow::Error> {
    let id = &id.replace('#', "%23");
    let token = env::var("CLASH_ROYALE_TOKEN").expect("Expected CLASH_ROYALE_TOKEN token");
    let token = format!("Bearer {}", token);
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, token.parse()?);
    let url = format!("https://api.clashroyale.com/v1/players/{}/battlelog", id);
    let client = reqwest::Client::new();
    let response = client.get(&url).headers(headers).send()
        .await?.text().await?;
    let response_json: Value = serde_json::from_str(&response)?;
    let mut ret = Vec::new();
    if let Some(battles) = response_json.as_array() {
        for battle in battles {
            let battle = battle.as_object().ok_or(anyhow::anyhow!("Unknown API error"))?;
            let time_string = battle["battleTime"].as_str().ok_or(anyhow::anyhow!("Unknown API error"))?;
            let time = chrono::NaiveDateTime::parse_from_str(time_string, "%Y%m%dT%H%M%S%.3fZ")?.timestamp();
            let opponent_object = battle["opponent"][0].as_object().ok_or(anyhow::anyhow!("Unknown API error"))?;
            let player = battle["team"][0].as_object().ok_or(anyhow::anyhow!("Unknown API error"))?;
            let opponent = opponent_object["name"].as_str().ok_or(anyhow::anyhow!("Unknown API error"))?.to_string();
            let mut opponent_crowns = 0;
            if opponent_object.contains_key("crowns") {
                opponent_crowns = opponent_object["crowns"].as_i64().unwrap_or(-1);
            }
            let mut victory = false;
            let mut crowns = 0;
            if player.contains_key("crowns") {
                crowns = player["crowns"].as_i64().unwrap_or(-1);
                victory = crowns > opponent_crowns;
            }
            let crowns: u8 = crowns.try_into()?;
            let opponent_crowns: u8 = opponent_crowns.try_into()?;
            ret.push(Battle { opponent, time, victory, crowns, opponent_crowns} )
        }
    }
    Ok(ret)
}
