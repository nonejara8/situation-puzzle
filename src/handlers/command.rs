use crate::models::{ChatCompletionMessage, Role};
use serenity::all::CommandInteraction;
use serenity::builder::{
    CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use serenity::model::application::ButtonStyle;
use serenity::model::user::User;
use serenity::prelude::*;

use crate::handlers::Bot;

pub async fn handle_command(ctx: Context, command: CommandInteraction, bot: &Bot) {
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

                if res.starts_with("正解です。") {
                    let builder = create_result_message(&command.user, &res, bot).await;

                    if let Err(e) = command.create_response(&ctx.http, builder).await {
                        println!("Error sending interaction response: {:?}", e);
                    }
                } else {
                    respond_to_command(&ctx, &command, res).await;
                }
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

async fn create_result_message(
    user: &User,
    description: &str,
    bot: &Bot,
) -> CreateInteractionResponse {
    let next_button = CreateButton::new("next_button")
        .label("次の問題に進む")
        .style(ButtonStyle::Primary);

    let cancel_button = CreateButton::new("cancel_button")
        .label("終了する")
        .style(ButtonStyle::Danger);

    let action_row = CreateActionRow::Buttons(vec![next_button, cancel_button]);

    let display_name = match user.global_name.clone() {
        Some(name) => name,
        None => user.name.clone(),
    };

    let mut scores = bot.scores.lock().await;

    if scores.contains_key(&display_name) {
        let score = scores.get_mut(&display_name).unwrap();
        *score += 1;
    } else {
        scores.insert(display_name.clone(), 1);
    }

    let mut message = "おめでとうございます🎉\n".to_string();
    message.push_str(&format!("{}さん　正解です！\n\n", user.mention()));
    message.push_str("問題のストーリー\n");
    message.push_str(description);

    let mut sorted_scores: Vec<_> = scores.iter().collect();
    sorted_scores.sort_by(|a, b| b.1.cmp(a.1));

    let fields: Vec<(String, String, bool)> = sorted_scores
        .iter()
        .map(|(user, score)| ((*user).clone(), format!("{}問正解", score), false))
        .collect();

    let embed = CreateEmbed::new()
        .color(0x00ff00)
        .description(message)
        .fields(fields);

    CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .embeds(vec![embed])
            .components(vec![action_row]),
    )
}

async fn respond_to_command(ctx: &Context, command: &CommandInteraction, response_content: String) {
    let data = CreateInteractionResponseMessage::new().content(response_content);
    let builder = CreateInteractionResponse::Message(data);

    if let Err(why) = command.create_response(&ctx.http, builder).await {
        println!("Cannot respond to slash command: {}", why);
        println!("command.data: {:?}", command.data);
    }
}
