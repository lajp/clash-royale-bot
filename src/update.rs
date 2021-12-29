use std::env;
use crate::Database;
use serenity::http::Http;
use crate::clashroyale::api::*;
use tracing::info;

lazy_static::lazy_static! {
    static ref BORING_CHESTS: Vec<&'static str> = vec!["Silver Chest", "Golden Chest", "Gold Crate", "Plentiful Gold Crate"];
}


pub async fn update_everything(http: &Http, db: Database) -> Result<(), anyhow::Error> {
    info!("Starting full-update");
    let startime = std::time::Instant::now();
    let notfication_channel_id = env::var("NOTIFICATION_CHANNEL_ID").expect("NOTIFICATION_CHANNEL_ID not defined in .env")
        .parse::<u64>().expect("Invalid NOTIFICATION_CHANNEL_ID");
    let notfication_channel = http.get_channel(notfication_channel_id).await?;
    let players = db.get_all_players().await?;
    info!("Starting the update of {} players!", players.len());
    for player in players {
        let mut battles = get_player_battles(&player.clash_royale_id).await?;
        battles.retain(|b| b.time > player.last_updated && b.victory);
        if battles.is_empty() {
            continue;
        }
        for battle in battles {
            notfication_channel.id().send_message(http, |m| {
                m.content(format!("<@{}> Onnittelut {}-{} voitostasi pelaajaa \"{}\" (nobo) vastaan!!!", player.discord_id, battle.crowns,
                battle.opponent_crowns, battle.opponent))
            }).await?;
        }
        let mut chests = get_player_chests(&player.clash_royale_id).await?;
        chests.retain(|c| c.index < 10 && !BORING_CHESTS.contains(&c.name.as_str()));
        for chest in chests {
            notfication_channel.id().send_message(http, |m| {
                m.content(format!("<@{}> Tulet saamaan chestin \"{}\" {} chestin päästä!!!!", player.discord_id, chest.name, chest.index))
            }).await?;
        }
        db.update_player(player).await?;
    }
    info!("Finished update in {} ms", startime.elapsed().as_millis());
    Ok(())
}
