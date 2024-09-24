use reqwest::Client;
use serde_json::{json, Value};


pub struct OpenAIClient {
    pub api_key: String,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
    }

    pub async fn send_request(
        &self,
        messages: &[ChatCompletionMessage],
    ) -> Result<String, anyhow::Error> {
        let client = Client::new();
        let body = json!({
            "model": "gpt-4o-mini",
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
                .unwrap()
                .get(0)
                .unwrap()
                .get("message")
                .unwrap()
                .get("content")
                .unwrap()
                .as_str()
                .unwrap()
                .replace("\"", "");
            Ok(message.to_string())
        } else {
            Err(anyhow::anyhow!(
                "エラーが発生しました: ステータスコード {}",
                status
            ))
        }
    }
}
