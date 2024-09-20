use tokio::sync::Mutex;

use serenity::all::{GuildId, Interaction, UserId};
use serenity::async_trait;
use serenity::builder::{CreateCommand, CreateCommandOption};

use std::collections::HashMap;
use tracing::info;

use crate::api::OpenAIClient;
use crate::models::ChatCompletionMessage;

use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::handlers::{handle_command, handle_component, handle_message};

pub struct Bot {
    pub discord_guild_id: GuildId,
    pub join_users: Mutex<Vec<UserId>>,
    pub openai_client: OpenAIClient,
    pub scores: Mutex<HashMap<String, u32>>,
    pub messages: Mutex<Vec<ChatCompletionMessage>>,
}

impl Bot {
    pub fn new(discord_guild_id: GuildId, openai_api_key: String) -> Self {
        Self {
            discord_guild_id,
            join_users: Mutex::new(vec![]),
            openai_client: OpenAIClient::new(openai_api_key),
            scores: Mutex::new(HashMap::new()),
            messages: Mutex::new(vec![]),
        }
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let commands = vec![
            CreateCommand::new("play").description("ゲームスタート"), // ゲームスタート
            CreateCommand::new("join").description("参加"),           // 参加
            CreateCommand::new("question")
                .description("質問を送信します")
                .add_option(
                    CreateCommandOption::new(
                        serenity::all::CommandOptionType::String,
                        "q",
                        "質問の内容を入力してください",
                    )
                    .max_length(100)
                    .required(true),
                ),
            CreateCommand::new("answer")
                .description("回答を送信します")
                .add_option(
                    CreateCommandOption::new(
                        serenity::all::CommandOptionType::String,
                        "a",
                        "回答の内容を入力してください",
                    )
                    .max_length(100)
                    .required(true),
                ),
            CreateCommand::new("giveup").description("ゲームを終了します"),
        ];

        let commands = &self
            .discord_guild_id
            .set_commands(&ctx.http, commands)
            .await
            .unwrap();

        info!("Registered commands: {:#?}", commands);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => handle_command(ctx, command, self).await,
            Interaction::Component(component) => handle_component(ctx, component, self).await,
            _ => (),
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        handle_message(ctx, msg).await;
    }
}
