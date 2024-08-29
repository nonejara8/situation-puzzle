mod handlers;

use std::sync::Mutex;

use anyhow::Context as _;
use serenity::all::{CommandInteraction, GuildId, Interaction, UserId};
use serenity::async_trait;
use serenity::builder::{
    CreateActionRow, CreateButton, CreateCommand, CreateCommandOption, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};

use serenity::model::application::ButtonStyle;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::info;

struct Bot {
    discord_guild_id: GuildId,
    join_users: Mutex<Vec<UserId>>,
}

impl Bot {
    fn new(discord_guild_id: GuildId) -> Self {
        Self {
            discord_guild_id,
            join_users: Mutex::new(vec![]),
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
            CreateCommand::new("show").description("参加者を表示"),    // 参加者を表示
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
            // Interaction::Component(component) => handle_component(ctx, component).await,
            _ => (),
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        handlers::handle_message(ctx, msg).await;
    }
}

async fn handle_command(ctx: Context, command: CommandInteraction, bot: &Bot) {
    let response_content = match command.data.name.as_str() {
        "play" => {
            let button = CreateButton::new("button_id")
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

            "buttonが押されました".to_owned()
        }
        "join" => {
            let user_id = command.user.id;
            let user_name = command.user.name.clone();
            let mut join_users = bot.join_users.lock().unwrap();
            join_users.push(user_id);
            format!(
                "{} さん(ID: {})が参加しました。\n現在の参加者数は{}人です。",
                user_name,
                user_id,
                join_users.len()
            )
        }
        "show" => {
            format!("{:?}", bot.join_users)
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

            format!("{}が選択されました", mode)
        }
        _ => "コマンドが見つかりません".to_owned(),
    };

    let data = CreateInteractionResponseMessage::new().content(response_content);
    let builder = CreateInteractionResponse::Message(data);

    if let Err(why) = command.create_response(&ctx.http, builder).await {
        println!("Cannot respond to slash command: {}", why);
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

    let client = Client::builder(&token, intents)
        .event_handler(Bot::new(GuildId::new(
            discord_guild_id.parse::<u64>().unwrap(),
        )))
        .await
        .expect("Err creating client");

    Ok(client.into())
}
