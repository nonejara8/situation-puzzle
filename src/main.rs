mod api;
mod handlers;

use tokio::sync::Mutex;

use anyhow::Context as _;
use serenity::all::{CommandInteraction, ComponentInteraction, GuildId, Interaction, UserId};
use serenity::async_trait;
use serenity::builder::{
    CreateActionRow, CreateButton, CreateCommand, CreateCommandOption, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
};

use serenity::model::application::ButtonStyle;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::info;

use crate::api::OpenAIClient;
struct Bot {
    discord_guild_id: GuildId,
    join_users: Mutex<Vec<UserId>>,
    openai_api_key: String,
}

impl Bot {
    fn new(discord_guild_id: GuildId, openai_api_key: String) -> Self {
        Self {
            discord_guild_id,
            join_users: Mutex::new(vec![]),
            openai_api_key,
        }
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let commands = vec![
            CreateCommand::new("play").description("参加⇔退出ボタン"), // 参加⇔退出ボタン
            CreateCommand::new("start").description("ゲームスタート"), // ゲームスタート
            CreateCommand::new("collector").description("コレクター"), // コレクター
            CreateCommand::new("join").description("参加"),            // 参加
            CreateCommand::new("question")
                .description("質問を送信します")
                .add_option(
                    CreateCommandOption::new(
                        serenity::all::CommandOptionType::String,
                        "質問の内容",
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
                        "回答の内容",
                        "回答の内容を入力してください",
                    )
                    .max_length(100)
                    .required(true),
                ),
            CreateCommand::new("typing")
                .description("typingのコマンドです")
                .add_option(CreateCommandOption::new(
                    serenity::all::CommandOptionType::Boolean,
                    "mode",
                    "ソロモードはtrue、マルチモードはfalse",
                )),
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
            Interaction::Component(component) => handle_component(ctx, component).await,
            _ => (),
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        handlers::handle_message(ctx, msg).await;
    }
}

async fn handle_command(ctx: Context, command: CommandInteraction, bot: &Bot) {
    match command.data.name.as_str() {
        "play" => {
            let button = CreateButton::new("button_1")
                .label("Click me!")
                .style(ButtonStyle::Primary);

            let action_row = CreateActionRow::Buttons(vec![button]);

            let response = CreateInteractionResponseMessage::new()
                .content("Here is a button!")
                .components(vec![action_row]);

            let builder = CreateInteractionResponse::Message(response);

            if let Err(e) = command.create_response(&ctx.http, builder).await {
                println!("Error sending interaction response: {:?}", e);
            }

            // フォローアップメッセージを送信
            if let Err(e) = command
                .create_followup(
                    &ctx.http,
                    CreateInteractionResponseFollowup::new().content("buttonが押されました"),
                )
                .await
            {
                println!("Error sending follow-up message: {:?}", e);
            }
        }
        "join" => {
            let user_id = command.user.id;
            let user_name = command.user.name.clone();
            let mut join_users = bot.join_users.lock().await;
            join_users.push(user_id);
            let response_content = format!(
                "{} さん(ID: {})が参加しました。\n現在の参加者数は{}人です。\nゲームを開始するには\\startを入力してください",
                user_name,
                user_id,
                join_users.len()
            );

            respond_to_command(&ctx, &command, response_content).await;
        }
        "start" => {
            let client = OpenAIClient::new(bot.openai_api_key.clone());
            let response = client.send_request().await;
            let response_content = "問題です".to_string();
            respond_to_command(&ctx, &command, response_content).await;
            // let client = OpenAIClient::new(bot.openai_api_key.clone());
            // let request = ChatCompletionRequest::new(
            //     "gpt-4o-mini".to_string(),
            //     vec![ChatCompletionMessage {
            //         role: MessageRole::system,
            //         content: Content::Text("あなた  はゲームのマスターです。".to_owned()),
            //     }],
            // );

            // let response: Result<ChatCompletionResponse, anyhow::Error> =
            //     client.post(&request).await;
            // match response {
            //     Ok(res) => res.choices[0].to_owned(),
            //     Err(e) => {
            //         println!("エラーが発生しました: {:?}", e);
            //         "エラーが発生しました".to_owned()
            //     }
            // }
        }
        "typing" => {
            let argument = command
                .data
                .options
                .iter()
                .find(|opt| opt.name == "mode")
                .cloned();

            let value = argument.unwrap().value;
            let mode = value.as_bool().unwrap();

            let response_content = format!("{}が選択されました", mode);

            respond_to_command(&ctx, &command, response_content).await;
        }
        _ => {}
    };
}

async fn respond_to_command(ctx: &Context, command: &CommandInteraction, response_content: String) {
    let data = CreateInteractionResponseMessage::new().content(response_content);
    let builder = CreateInteractionResponse::Message(data);

    if let Err(why) = command.create_response(&ctx.http, builder).await {
        println!("Cannot respond to slash command: {}", why);
        println!("command.data: {:?}", command.data);
    }
}

async fn handle_component(ctx: Context, component: ComponentInteraction) {
    let response_content = match component.data.custom_id.as_str() {
        "button_1" => "ボタン1が押されました",
        "button_2" => "ボタン2が押されました",
        _ => "未知のボタンが押されました",
    };

    let builder = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new().content(response_content),
    );

    if let Err(why) = component.create_response(&ctx.http, builder).await {
        println!("Cannot respond to component interaction: {}", why);
    }
}

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
