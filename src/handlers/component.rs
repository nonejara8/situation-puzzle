use serenity::all::ComponentInteraction;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::prelude::*;

use crate::handlers::Bot;
use crate::utils::question_generator::generate_question_builder;

pub async fn handle_component(ctx: Context, component: ComponentInteraction, bot: &Bot) {
    match component.data.custom_id.as_str() {
        "next_button" => next_button(component, ctx, bot).await,
        "cancel_button" => finish_button(component, ctx, bot).await,
        _ => unknown_component(component, ctx).await,
    };
}

async fn next_button(component: ComponentInteraction, ctx: Context, bot: &Bot) {
    let builder = generate_question_builder(bot).await;
    if let Err(why) = component.create_response(&ctx.http, builder).await {
        println!("Cannot respond to slash command: {}", why);
        println!("command.data: {:?}", component.data);
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
