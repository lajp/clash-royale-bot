pub mod schema;
pub mod models;
pub mod database;
pub mod clashroyale;
pub mod update;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;

#[macro_use]
extern crate diesel;

use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::*;
use serenity::http::Http;
use serenity::model::gateway::Ready;
use serenity::model::interactions::application_command::{
    ApplicationCommand, ApplicationCommandOptionType,
};
use serenity::model::interactions::{Interaction, InteractionResponseType};
use serenity::prelude::*;
use tracing::{error, info};
use clokwerk::{Scheduler, TimeUnits};

use database::*;
pub struct ShardManagerContainer;


impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let response = match command.data.name.as_str() {
                "clashroyale" => {
                    let mut r = "???".to_string();
                    for a in &command.data.options {
                        match a.name.as_str() {
                            "userid" => {
                                let data = ctx.data.read().await;
                                let db = data.get::<Database>().unwrap();
                                let cr_id = a.value.as_ref().unwrap().as_str().unwrap();
                                r = match db.get_player(command.user.id.0.try_into().unwrap(), cr_id).await {
                                    Ok(_) => String::from("Player already exist in the database, not adding"),
                                    Err(_) => {
                                        db.new_player(command.user.id.0.try_into().unwrap(), cr_id).await.unwrap();
                                        String::from("Player succesfully added to the database!")
                                    }
                                };
                            }
                            _ => unreachable!()
                        };
                    }
                    r
                }
                _ => unreachable!()
            };
            command
                .create_interaction_response(&ctx.http, |r| {
                    r
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|m| m.content(response))
                })
                .await
                .unwrap()
        }
    }
    async fn ready(&self, ctx: Context, ready: Ready) {
        ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command
                        .name("clashroyale")
                        .description("Lisää käyttäjäsi bottiin")
                        .create_option(|option| {
                            option
                                .name("userid")
                                .description("Sinun clash-royale ID")
                                .required(true)
                                .kind(ApplicationCommandOptionType::String)
                        })
                })
        })
        .await
        .unwrap();
        info!("Connected as {}", ready.user.name);
    }
}

#[group]
struct General;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect(".env required");

    tracing_subscriber::fmt::init();

    let database = Database::new().await;

    let token = env::var("DISCORD_TOKEN").expect("No DISCORD_TOKEN in .env");
    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("No APPLICATION_ID in .env")
        .parse()
        .expect("Invalid APPLICATION_ID provided");
    let http = Http::new_with_token(&token);

    let (owner, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owner).prefix("!"))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .application_id(application_id)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Error while creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Database>(database.clone());
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut scheduler = Scheduler::with_tz(chrono::Local);

    scheduler.every(30.second()).run(move || {
        if let Err(e) = runtime.block_on(update::update_everything(&http, database.clone())) {
            error!("Update failed!: {}", e);
        }
    });

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl-c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
