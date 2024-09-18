// 設定関連の構造体や設定読み込みロジック
// 環境変数や設定ファイルからの読み込みを行う

use serenity::model::id::GuildId;
use shuttle_runtime::SecretStore;

#[derive(Clone)]
pub struct Config {
    pub discord_token: String,
    pub discord_guild_id: GuildId,
    pub openai_api_key: String,
}

impl Config {
    // Get the discord token set in `Secrets.toml`
    pub async fn from_secrets(secrets: &SecretStore) -> Self {
        Self {
            discord_token: secrets
                .get("DISCORD_TOKEN")
                .expect("'DISCORD_TOKEN' was not found"),
            discord_guild_id: GuildId::new(
                secrets
                    .get("DISCORD_GUILD_ID")
                    .expect("'DISCORD_GUILD_ID' was not found")
                    .parse::<u64>()
                    .expect("DISCORD_GUILD_ID parse failed"),
            ),
            openai_api_key: secrets
                .get("OPENAI_API_KEY")
                .expect("'OPENAI_API_KEY' was not found"),
        }
    }
}
