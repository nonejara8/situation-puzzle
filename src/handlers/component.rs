use serenity::all::ComponentInteraction;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::prelude::*;

use crate::handlers::Bot;
use crate::models::State;
use crate::utils::question_generator::generate_question_builder;

pub async fn handle_component(ctx: Context, component: ComponentInteraction, bot: &Bot) {
    match component.data.custom_id.as_str() {
        "next_button" => next_button(component, ctx, bot).await,
        "cancel_button" => finish_button(component, ctx, bot).await,
        _ => unknown_component(component, ctx).await,
    };
}

async fn next_button(component: ComponentInteraction, ctx: Context, bot: &Bot) {
    if !matches!(*bot.state.lock().await, State::Waiting) {
        respond_to_component_ephemeral(
            &ctx,
            &component,
            "実行するタイミングが正しくありません".to_string(),
        )
        .await;
        return;
    }

    let builder = generate_question_builder(bot).await;
    if let Err(why) = component.create_response(&ctx.http, builder).await {
        println!("次の問題の生成に失敗しました: {}", why);
        println!("component.data: {:?}", component.data);
        return;
    }

    bot.set_state(State::Playing).await;
}

async fn finish_button(component: ComponentInteraction, ctx: Context, bot: &Bot) {
    if !matches!(*bot.state.lock().await, State::Waiting) {
        respond_to_component_ephemeral(
            &ctx,
            &component,
            "実行するタイミングが正しくありません".to_string(),
        )
        .await;
        return;
    }

    respond_to_component(&ctx, &component, "ゲームを終了します".to_string()).await;
    bot.initialize().await;
}

async fn unknown_component(component: ComponentInteraction, ctx: Context) -> () {
    let msg: &str = "未知のコンポーネントが呼ばれました";

    let builder =
        CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(msg));

    if let Err(why) = component.create_response(&ctx.http, builder).await {
        println!("Cannot respond to component interaction: {}", why);
    }
}

async fn respond_to_component(
    ctx: &Context,
    component: &ComponentInteraction,
    response_content: String,
) {
    let data = CreateInteractionResponseMessage::new().content(response_content);
    let builder = CreateInteractionResponse::Message(data);

    if let Err(why) = component.create_response(&ctx.http, builder).await {
        println!("コンポーネントの返答に失敗しました: {}", why);
        println!("component.data: {:?}", component.data);
    }
}

async fn respond_to_component_ephemeral(
    ctx: &Context,
    component: &ComponentInteraction,
    response_content: String,
) {
    let data = CreateInteractionResponseMessage::new()
        .content(response_content)
        .ephemeral(true);
    let builder = CreateInteractionResponse::Message(data);

    if let Err(why) = component.create_response(&ctx.http, builder).await {
        println!("コンポーネントの返答に失敗しました: {}", why);
        println!("component.data: {:?}", component.data);
    }
}
