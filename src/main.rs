mod api;
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



#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let discord_guild_id = secrets
        .get("DISCORD_GUILD_ID")
        .context("'DISCORD_GUILD_ID' was not found")?;

    let openai_api_key = secrets
        .get("OPENAI_API_KEY")
        .context("'OPENAI_API_KEY' was not found")?;

    let client = Client::builder(&token, intents)
        .event_handler(Bot::new(
            GuildId::new(discord_guild_id.parse::<u64>().unwrap()),
            openai_api_key.to_string(),
        ))
        .await
        .expect("Err creating client");

    Ok(client.into())
}
