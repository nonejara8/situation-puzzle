use anyhow::Context as _;
use serenity::all::{GuildId, Interaction};
use serenity::async_trait;
use serenity::builder::{
    CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::{error, info};

struct Bot {
    discord_guild_id: GuildId,
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let commands = vec![
            CreateCommand::new("hello").description("helloのコマンドです"),
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
        if let Interaction::Command(command) = interaction {
            let response_content = match command.data.name.as_str() {
                "hello" => "world!".to_owned(),
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
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            }
        }

        if msg.content == "!typing" {
            if let Err(e) = msg.channel_id.broadcast_typing(&ctx.http).await {
                error!("Error broadcasting typing: {:?}", e);
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            if let Err(e) = msg.channel_id.say(&ctx.http, "〇〇が入力中").await {
                error!("Error sending message: {:?}", e);
            }
        }
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
        .event_handler(Bot {
            discord_guild_id: GuildId::new(discord_guild_id.parse::<u64>().unwrap()),
        })
        .await
        .expect("Err creating client");

    Ok(client.into())
}
