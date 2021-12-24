#[derive(Queryable, Clone, Debug)]
pub struct Player {
    pub id: i32,
    pub discord_id: i64,
    pub clash_royale_id: String,
    pub last_updated: i64,
}

use crate::schema::Players;

#[derive(Insertable)]
#[table_name="Players"]
pub struct NewPlayer {
    pub discord_id: i64,
    pub clash_royale_id: String,
    pub last_updated: i64,
}
