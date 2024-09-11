use anyhow::Error;
use reqwest::{Client, Method, RequestBuilder};
use serde_json::{json, Value};
const API_URL_V1: &str = "https://api.openai.com/v1";

pub struct OpenAIClient {
    pub api_key: String,
    pub api_base: String,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            api_base: API_URL_V1.to_string(),
        }
    }

    pub async fn send_request(&self) -> Result<String, anyhow::Error> {
        let client = Client::new();
        let body = json!({
            "model": "gpt-4o-mini",
            "messages": [{"role": "user", "content": "Hello, how are you?"}]
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
                .unwrap();
            Ok(message.to_string())
        } else {
            Err(anyhow::anyhow!(
                "エラーが発生しました: ステータスコード {}",
                status
            ))
        }
    }
}
