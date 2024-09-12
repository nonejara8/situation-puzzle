#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let commands = vec![CreateCommand::new("typing")
            .description("typingのコマンドです")
            .add_option(CreateCommandOption::new(
                serenity::all::CommandOptionType::Boolean,
                "mode",
                "ソロモードはtrue、マルチモードはfalse",
            ))];

        let commands = &self
            .discord_guild_id
            .set_commands(&ctx.http, commands)
            .await
            .unwrap();

        info!("Registered commands: {:#?}", commands);
    }
}

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
        "typing" => {
            let argument = command
                .data
                .options
                .iter()
                .find(|opt| opt.name == "mode")
                .cloned();

            let value = argument.unwrap().value;
            let mode = value.as_bool().unwrap();

            let response_content = format!("{}が選択されました", mode);

            respond_to_command(&ctx, &command, response_content).await;
        }
        _ => {}
    };
}
