use serenity::all::ComponentInteraction;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::prelude::*;

pub async fn handle_component(ctx: Context, component: ComponentInteraction) {
    let response_content = match component.data.custom_id.as_str() {
        "next_button" => "次の問題に進みます",
        "cancel_button" => "ゲームを終了します",
        _ => "未知のボタンが押されました",
    };

    let builder = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new().content(response_content),
    );

    if let Err(why) = component.create_response(&ctx.http, builder).await {
        println!("Cannot respond to component interaction: {}", why);
    }
}
