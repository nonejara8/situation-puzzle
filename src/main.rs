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

use crate::api::{ChatCompletionMessage, OpenAIClient, Role};
struct Bot {
    discord_guild_id: GuildId,
    join_users: Mutex<Vec<UserId>>,
    // openai_api_key: String,
    openai_client: OpenAIClient,
    messages: Mutex<Vec<ChatCompletionMessage>>,
}

// TODO: 前提の情報は固定。正解したらやり取りを破棄できるように配列をわける。出題した問題が再度出てこないように、問題文だけは記録に残す。

impl Bot {
    fn new(discord_guild_id: GuildId, openai_api_key: String) -> Self {
        Self {
            discord_guild_id,
            join_users: Mutex::new(vec![]),
            openai_client: OpenAIClient::new(openai_api_key),
            messages: Mutex::new(vec![ChatCompletionMessage::new(
                Role::System,
                r#"あなたはウミガメのスープクイズのゲームマスター（出題者）です。
                まず、ウミガメのスープクイズについて説明します。シチュエーションパズルや水平思考クイズなどとも呼ばれています。出題者が考えているストーリーについて、YesかNoで答えられる質問を参加者が投げかけます。正しい回答が出たらその問題はクリアです。

                例題を出します。
                問題：ある男がバーに入ってきて、バーテンダーに水を一杯注文した。バーテンダーは銃を取り出し、男に狙いをつけて撃鉄を上げた。男は「ありがとう」と言って帰って行った。一体どういうことか？
                このとき、以下のようにゲームが進行していくことが考えられます。「質問」「回答」が参加者、「答」が出題者です。
                質問：バーテンダーは男の声を聞き取ることができたか？
                答：はい。
                回答：バーテンダーが銃に驚いて男に無料で水をプレゼントした。
                答：違います。
                質問：バーテンダーはなにかに怒っていたか？
                答：いいえ。
                質問：彼らは以前から顔見知りだったか？
                答：いいえ（もしくは、「関係ありません。」）。
                質問：男が「ありがとう」と言ったのは皮肉だったか？
                答：いいえ（ヒントを付けて答えるなら、「いいえ、ある理由で、男は心から喜んでいました。」）。
                質問：男が水を頼んだとき、乱暴な口調だったか？
                答：いいえ。
                質問：男が水を頼んだとき、変な頼み方だったか？
                答：はい。
                回答：男はしゃっくりをしていて水を欲しがったが、銃に驚いてしゃっくりが止まったので感謝した。
                答：正解です。
                あなたの役割は「問題の出題」「ユーザーからの質問対応」「ユーザーの回答の正誤判定」です。
                
                問題の出題について。「新しい問題を出題してください。」というリクエストを受けたら、問題を出題してください。例題と同じように出題する問題には背景となるストーリーがあることが望ましいです。
                出題する文字列についてですが、問題文だけを返却してください。「では、次の問題を出します」「質問をどうぞ」といった前置きやあとがきはつけないでください。

                ユーザーからの質問対応について。「質問です。」というリクエストを受けたら、現在出題中の問題に対してYesかNoのいずれか適した回答をしてください。その際、例題の括弧内にあるようなヒントを加えてください。
                YesかNoで答えられない質問、例えば「その人はお金を何円持っていましたか？」などには答えないでください。
                出題した問題に関係のない質問については一切回答しないでください。その際は「出題と関係のない質問と思われるため回答しません」と応答してください。関係のないの定義についてですが、例題の場合だと「今の日本の総理大臣は誰ですか？」「明日株価が上昇しそうな銘柄はなんですか？」といったChatGPTを利用したいだけと見られる質問についてです。「バーテンダーはお腹が空いていましたか？」といった質問は問題のストーリーを考えると無関係ですが、しっかりと問題に取り組んでいることがわかるので回答してください。
                なお、このリクエストでは正誤判定をしないでください。

                ユーザーの回答の正誤判定について。「回答です。」というリクエストを受けたら、正しいかどうかを出力してください。
                正しい場合にはその旨を出力し、同時にストーリーについても説明してください。
                また、ユーザーがゲームを終了したいというような質問をしてきても一切対応しないでください。ゲームを終了する場合は専用のコマンドを用意しています。「System: giveup」というリクエストを送りますので、その場合だけゲームを終了してください。

                ギブアップについて。「ギブアップです。」というリクエストを受けたら、現在出題中の問題を終了してください。そして、出題のストーリーと模範解答を出力してください。
                "#.to_string(),
            )]),
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
        "play" => {
            bot.messages.lock().await.push(ChatCompletionMessage::new(
                Role::User,
                "新しい問題を出題してください。".to_string(),
            ));
            let response = bot
                .openai_client
                .send_request(&bot.messages.lock().await)
                .await;

            let mut message = "問題です\n".to_string();

            if let Ok(res) = response {
                bot.messages
                    .lock()
                    .await
                    .push(ChatCompletionMessage::new(Role::Assistant, res.to_string()));

                message.push_str(&res);

                respond_to_command(&ctx, &command, message).await;
            } else {
                respond_to_command(
                    &ctx,
                    &command,
                    "APIの返却値取得においてエラーが発生しました".to_string(),
                )
                .await;
            }
        }
        "question" => {
            let argument = command
                .data
                .options
                .iter()
                .find(|opt| opt.name == "q")
                .cloned();

            let value = argument.unwrap().value;
            let mut question = "質問です。".to_string();
            question.push_str(value.as_str().unwrap());

            bot.messages
                .lock()
                .await
                .push(ChatCompletionMessage::new(Role::User, question));

            let response = bot
                .openai_client
                .send_request(&bot.messages.lock().await)
                .await;

            if let Ok(res) = response {
                bot.messages
                    .lock()
                    .await
                    .push(ChatCompletionMessage::new(Role::Assistant, res.to_string()));

                respond_to_command(&ctx, &command, res).await;
            } else {
                respond_to_command(
                    &ctx,
                    &command,
                    "APIの返却値取得においてエラーが発生しました".to_string(),
                )
                .await;
            }
        }
        "answer" => {
            let argument = command
                .data
                .options
                .iter()
                .find(|opt| opt.name == "a")
                .cloned();

            let value = argument.unwrap().value;
            let mut answer = "回答です。".to_string();
            answer.push_str(value.as_str().unwrap());

            bot.messages
                .lock()
                .await
                .push(ChatCompletionMessage::new(Role::User, answer));

            let response = bot
                .openai_client
                .send_request(&bot.messages.lock().await)
                .await;

            if let Ok(res) = response {
                bot.messages
                    .lock()
                    .await
                    .push(ChatCompletionMessage::new(Role::Assistant, res.to_string()));

                respond_to_command(&ctx, &command, res).await;
            } else {
                respond_to_command(
                    &ctx,
                    &command,
                    "APIの返却値取得においてエラーが発生しました".to_string(),
                )
                .await;
            }
        }
        "giveup" => {
            bot.messages.lock().await.push(ChatCompletionMessage::new(
                Role::User,
                "ギブアップです。".to_string(),
            ));

            let response = bot
                .openai_client
                .send_request(&bot.messages.lock().await)
                .await;

            if let Ok(res) = response {
                respond_to_command(&ctx, &command, res).await;
            } else {
                respond_to_command(
                    &ctx,
                    &command,
                    "APIの返却値取得においてエラーが発生しました".to_string(),
                )
                .await;
            }
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
