async fn handle_command(ctx: Context, command: CommandInteraction, bot: &Bot) {
    match command.data.name.as_str() {
        "demo" => {
            let button = CreateButton::new("button_1")
                .label("Click me!")
                .style(ButtonStyle::Primary);

            let action_row = CreateActionRow::Buttons(vec![button]);

            let response = CreateInteractionResponseMessage::new()
                .content("Here is a button!")
                .components(vec![action_row]);

            let builder = CreateInteractionResponse::Message(response);

            if let Err(e) = command.create_response(&ctx.http, builder).await {
                println!("Error sending interaction response: {:?}", e);
            }

            // フォローアップメッセージを送信
            if let Err(e) = command
                .create_followup(
                    &ctx.http,
                    CreateInteractionResponseFollowup::new().content("buttonが押されました"),
                )
                .await
            {
                println!("Error sending follow-up message: {:?}", e);
            }
        }
        _ => {}
    };
}
