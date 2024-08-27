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
