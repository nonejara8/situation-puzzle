mod api;
mod config;
mod constants;
mod handlers;
mod models;
mod utils;

use serenity::prelude::*;
use shuttle_runtime::SecretStore;

use crate::config::Config;

use crate::handlers::Bot;

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    let config = Config::from_secrets(&secrets).await;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&config.discord_token, intents)
        .event_handler(Bot::new(config.discord_guild_id, config.openai_api_key))
        .await
        .expect("Err creating client");

    Ok(client.into())
}
