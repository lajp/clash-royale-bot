use serenity::prelude::TypeMapKey;

use std::env;
use tracing::info;
use diesel::prelude::*;
use crate::models::*;
use diesel::sqlite::SqliteConnection;
use diesel::r2d2::{Pool, ConnectionManager};

#[derive(Clone)]
pub struct Database {
    database: Pool<ConnectionManager<SqliteConnection>>,
}

impl TypeMapKey for Database {
    type Value = Database;
}

impl Database {
    pub async fn new() -> Database {
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let manager = ConnectionManager::<SqliteConnection>::new(&database_url);
        let database = Pool::builder().build(manager).expect("Failed to creaTe connection pool");

        Self { database }
    }
    pub async fn get_player(&self, dc_id: i64, cr_id: &str) -> Result<Player, diesel::result::Error> {
        use crate::schema::Players::dsl::*;
        Players.filter(discord_id.eq(dc_id).and(clash_royale_id.eq(cr_id)))
            .first::<Player>(&self.database.get().unwrap())
    }
    pub async fn get_all_players(&self) -> Result<Vec<Player>, diesel::result::Error> {
        use crate::schema::Players::dsl::*;
        Players.load::<Player>(&self.database.get().unwrap())
    }
    pub async fn new_player(&self, dc_id: i64, cr_id: &str) -> Result<usize, diesel::result::Error> {
        info!("Adding player with crid: {} and dcid: {} to the database", cr_id, dc_id);
        use crate::schema::Players;
        let new_player = NewPlayer {
            last_updated: chrono::Utc::now().naive_local().timestamp(),
            discord_id: dc_id,
            clash_royale_id: String::from(cr_id),
        };

        diesel::insert_into(Players::table)
            .values(&new_player)
            .execute(&self.database.get().unwrap())
    }
    pub async fn update_player(&self, p: Player) -> Result<usize, diesel::result::Error> {
        use crate::schema::Players::dsl::*;
        diesel::update(Players.filter(discord_id.eq(p.discord_id).and(clash_royale_id.eq(p.clash_royale_id))))
                .set(last_updated.eq(chrono::Utc::now().naive_local().timestamp()))
                .execute(&self.database.get().unwrap())
    }
}
