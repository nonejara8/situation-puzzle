use serenity::all::Timestamp;
use serenity::builder::{CreateEmbed, CreateEmbedFooter, CreateMessage};
use serenity::model::channel::Message;
use serenity::prelude::*;
use tracing::error;

pub async fn handle_message(ctx: Context, msg: Message) {
    if msg.content == "!nurupo" {
        let res = format!(
            "{}",
            r"
```
　 　＿＿＿＿＿　　　　　 ／￣￣￣￣￣￣￣￣￣￣￣￣
　／:＼.＿＿＿＿＼ 　／　
　|:￣＼(∩( ;ﾟ∀ﾟ)　＜　　　ぬるぽぬるぽぬるぽーーー！
　|:　　 |:￣￣￣∪:|　　＼
　　　　　　　　　　　　　　　＼＿＿＿＿＿＿＿＿＿＿＿＿

　　
　　　　ﾊﾞﾀﾝｯ!!
　＿＿＿＿＿___
　|:￣＼　　　　 　＼ 　　＜ﾇﾙﾎﾟﾇﾙﾎﾟｰ!!
　|:　　 |:￣￣￣￣:|
```"
        );

        if let Err(e) = msg.channel_id.say(&ctx.http, res).await {
            error!("Error sending message: {:?}", e);
        }
    }

    if msg.content == "!ga" {
        let res = format!(
            "{}",
            r"
```
　　 （　・∀・）　　　|　|　ｶﾞｯ\n\
　　と　　　　）　 　 |　|\n\
　　　 Ｙ　/ノ　　　 人\n\
　　　　 /　）　 　 < 　>__Λ∩\n\
　　 ＿/し'　／／. Ｖ｀Д´）/\n\
　　（＿フ彡　　　　　 　　/\n\
```"
        );

        if let Err(e) = msg.channel_id.say(&ctx.http, res).await {
            error!("Error sending message: {:?}", e);
        }
    }

    if msg.content == "!embed" {
        // The create message builder allows you to easily create embeds and messages using a
        // builder syntax.
        // This example will create a message that says "Hello, World!", with an embed that has
        // a title, description, an image, three fields, and a footer.
        let footer = CreateEmbedFooter::new("This is a footer");
        let embed = CreateEmbed::new()
            .title("This is a title")
            .description("This is a description")
            .fields(vec![
                ("This is the first field", "This is a field body", true),
                ("This is the second field", "Both fields are inline", true),
            ])
            .field(
                "This is the third field",
                "This is not an inline field",
                false,
            )
            .footer(footer)
            // Add a timestamp for the current time
            // This also accepts a rfc3339 Timestamp
            .timestamp(Timestamp::now());
        let builder = CreateMessage::new().content("Hello, World!").embed(embed);

        let msg = msg.channel_id.send_message(&ctx.http, builder).await;

        if let Err(why) = msg {
            println!("Error sending message: {why:?}");
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
