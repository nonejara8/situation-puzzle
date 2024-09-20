use serenity::all::ComponentInteraction;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::prelude::*;

use crate::handlers::Bot;
use crate::models::{ChatCompletionMessage, Role};

pub async fn handle_component(ctx: Context, component: ComponentInteraction, bot: &Bot) {
    match component.data.custom_id.as_str() {
        "next_button" => next_button(component, ctx, bot).await,
        "cancel_button" => finish_button(component, ctx, bot).await,
        _ => unknown_component(component, ctx).await,
    };
}

async fn next_button(component: ComponentInteraction, ctx: Context, bot: &Bot) {
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

        let builder = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().content(message),
        );

        if let Err(why) = component.create_response(&ctx.http, builder).await {
            println!("Cannot respond to component interaction: {}", why);
        }
    }
}

async fn finish_button(component: ComponentInteraction, ctx: Context, bot: &Bot) {}

async fn unknown_component(component: ComponentInteraction, ctx: Context) -> () {
    let msg: &str = "未知のコンポーネントが呼ばれました";

    let builder =
        CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(msg));

    if let Err(why) = component.create_response(&ctx.http, builder).await {
        println!("Cannot respond to component interaction: {}", why);
    }
}
