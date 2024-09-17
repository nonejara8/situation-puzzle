use anyhow::Error;
use reqwest::{Client, Method, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
const API_URL_V1: &str = "https://api.openai.com/v1";

pub struct OpenAIClient {
    pub api_key: String,
    pub api_base: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionMessage {
    pub role: Role,
    pub content: String,
}

impl ChatCompletionMessage {
    pub fn new(role: Role, content: String) -> Self {
        Self { role, content }
    }
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            api_base: API_URL_V1.to_string(),
        }
    }

    pub async fn send_request(
        &self,
        messages: &[ChatCompletionMessage],
    ) -> Result<String, anyhow::Error> {
        let client = Client::new();
        let body = json!({
            "model": "gpt-4o-mini",
            "messages": messages
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
