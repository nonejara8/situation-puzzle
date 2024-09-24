use reqwest::Client;
use serde_json::{json, Value};

use crate::models::ChatCompletionMessage;

pub struct OpenAIClient {
    pub api_key: String,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub async fn send_request(
        &self,
        messages: &[ChatCompletionMessage],
    ) -> Result<String, anyhow::Error> {
        let client = Client::new();
        let body = json!({
            "model": "gpt-4o-mini",
            "messages": messages,
        });

        let request = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .build()?;

        let response = client.execute(request).await?;
        let status = response.status();
        let headers = response.headers().clone();
        let json: Value = response.json().await?;

        println!("Status: {}", status);
        println!("Headers: {:?}", headers);
        println!("Response JSON: {}", json);

        if status.is_success() {
            let message = json
                .get("choices")
                .and_then(|choices| choices.get(0))
                .and_then(|choice| choice.get("message"))
                .and_then(|message| message.get("content"))
                .and_then(|content| content.as_str())
                .ok_or_else(|| anyhow::anyhow!("レスポンスの形式が意図したものではありません"))?
                .to_string();
            Ok(message)
        } else {
            Err(anyhow::anyhow!(
                "エラーが発生しました: ステータスコード {}",
                status
            ))
        }
    }
}
