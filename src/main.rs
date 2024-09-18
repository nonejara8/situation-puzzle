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
struct Bot {
    discord_guild_id: GuildId,
    join_users: Mutex<Vec<UserId>>,
    // openai_api_key: String,
    openai_client: OpenAIClient,
    scores: Mutex<HashMap<String, u32>>,
    messages: Mutex<Vec<ChatCompletionMessage>>,
}

// TODO: 前提の情報は固定。正解したらやり取りを破棄できるように配列をわける。出題した問題が再度出てこないように、問題文だけは記録に残す。

impl Bot {
    fn new(discord_guild_id: GuildId, openai_api_key: String) -> Self {
        Self {
            discord_guild_id,
            join_users: Mutex::new(vec![]),
            openai_client: OpenAIClient::new(openai_api_key),
            scores: Mutex::new(HashMap::new()),
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

                問題文として適切ではない例もあげます。
                問題：ある男性が海岸で釣りをしていると、突然大きな波が押し寄せてきました。男性は波にさらわれ、自分の道具や魚を失ってしまいました。しかし、男性は驚いていませんでした。なぜでしょうか？
                模範解答：男性は波が来ることを予測しており、あらかじめ釣り道具や魚を安全な場所に移動させていたからです。
                これは、ストーリーがないので好ましくありません。自分の道具を失ってしまったという前提を回答が無視しているためです。
                問題だけでは推測できず、何回か質問することで回答できるような問題を考えてください。

                あなたの役割は「問題の出題」「ユーザーからの質問対応」「ユーザーの回答の正誤判定」です。
                問題の出題について。「新しい問題を出題してください。」というリクエストを受けたら、問題を出題してください。例題と同じように出題する問題には背景となるストーリーがあることが望ましいです。

                出題する文字列についてですが、問題文だけを返却してください。「では、次の問題を出します」「質問をどうぞ」といった前置きやあとがきはつけないでください。

                ユーザーからの質問対応について。「質問です。」というリクエストを受けたら、現在出題中の問題に対してYesかNoのいずれか適した回答をしてください。その際、例題の括弧内にあるようなヒントを加えてください。
                YesかNoで答えられない質問、例えば「その人はお金を何円持っていましたか？」などには答えないでください。
                出題した問題に関係のない質問については一切回答しないでください。その際は「出題と関係のない質問と思われるため回答しません」と応答してください。関係のないの定義についてですが、例題の場合だと「今の日本の総理大臣は誰ですか？」「明日株価が上昇しそうな銘柄はなんですか？」といったChatGPTを利用したいだけと見られる質問についてです。「バーテンダーはお腹が空いていましたか？」といった質問は問題のストーリーを考えると無関係ですが、しっかりと問題に取り組んでいることがわかるので回答してください。
                なお、このリクエストでは正誤判定をしないでください。

                ユーザーの回答の正誤判定について。「回答です。」というリクエストを受けたら、正しいかどうかを出力してください。
                正しい場合には「正解です。」と出力し、同時にストーリーについても説明してください。
                不正解の場合には「不正解です。」と出力してください。
                また、ユーザーがゲームを終了したいというような質問をしてきても一切対応しないでください。ゲームを終了する場合は専用のコマンドを用意しています。「System: giveup」というリクエストを送りますので、その場合だけゲームを終了してください。

                ギブアップについて。「ギブアップです。」というリクエストを受けたら、現在出題中の問題を終了してください。そして、出題のストーリーと模範解答を出力してください。
                "#.to_string(),
            )]),
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
