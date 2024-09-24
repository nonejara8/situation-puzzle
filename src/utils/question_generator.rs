use crate::handlers::Bot;
use crate::models::{ChatCompletionMessage, Role};

use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};

pub async fn generate_question_builder(bot: &Bot) -> CreateInteractionResponse {
    bot.reset_messages().await;
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

        let data = CreateInteractionResponseMessage::new().content(message);

        CreateInteractionResponse::Message(data)
    } else {
        let data = CreateInteractionResponseMessage::new()
            .content("APIの返却値取得においてエラーが発生しました".to_string());

        CreateInteractionResponse::Message(data)
    }
}
