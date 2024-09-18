mod api;
mod config;
mod handlers;

use tokio::sync::Mutex;

use anyhow::Context as _;
use serenity::all::{
    CommandInteraction, ComponentInteraction, CreateEmbedFooter, CreateMessage, GuildId,
    Interaction, UserId,
};
use serenity::async_trait;
use serenity::builder::{
    CreateActionRow, CreateButton, CreateCommand, CreateCommandOption, CreateEmbed,
    CreateInteractionResponse, CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
};
use serenity::model::application::ButtonStyle;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use std::collections::HashMap;
use tracing::info;

use crate::api::{ChatCompletionMessage, OpenAIClient, Role};

use crate::config::Config;


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
