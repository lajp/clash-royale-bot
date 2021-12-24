use std::env;
use crate::Database;
use serenity::http::Http;
use crate::clashroyale::api::*;
use tracing::info;


pub async fn update_everything(http: &Http, db: Database) {
    let boring_chests = vec!["Silver Chest", "Golden Chest", "Gold Crate", "Plentiful Gold Crate"];
    dotenv::dotenv().ok();
    let notfication_channel_id = env::var("NOTIFICATION_CHANNEL_ID").expect("NOTIFICATION_CHANNEL_ID not defined in .env")
        .parse::<u64>().expect("Invalid NOTIFICATION_CHANNEL_ID");
    let notfication_channel = http.get_channel(notfication_channel_id).await.unwrap();
    let players = db.get_all_players().await.unwrap();
    info!("Starting the update of {} players!", players.len());
    for player in players {
        let mut battles = get_player_battles(&player.clash_royale_id).await;
        battles.retain(|b| b.time > player.last_updated && b.victory);
        if battles.is_empty() {
            continue;
        }
        for battle in battles {
            notfication_channel.id().send_message(http, |m| {
                m.content(format!("<@{}> Onnittelut {}-{} voitostasi pelaajaa \"{}\" (nobo) vastaan!!!", player.discord_id, battle.crowns,
                battle.opponent_crowns, battle.opponent))
            }).await.unwrap();
        }
        let mut chests = get_player_chests(&player.clash_royale_id).await;
        chests.retain(|c| c.index < 10 && !boring_chests.contains(&c.name.as_str()));
        for chest in chests {
            notfication_channel.id().send_message(http, |m| {
                m.content(format!("<@{}> Tulet saamaan chestin \"{}\" {} chestin päästä!!!!", player.discord_id, chest.name, chest.index))
            }).await.unwrap();
        }
        db.update_player(player).await.unwrap();
    }
}
