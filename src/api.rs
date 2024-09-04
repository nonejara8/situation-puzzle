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

    pub async fn send_request(&self) -> Result<Value, reqwest::Error> {
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

        Ok(json)
    }

    async fn build_request(&self, method: Method) -> reqwest::RequestBuilder {
        let url = format!("{}/chat/completions", self.api_base);
        let client = Client::new();

        let request = client
            .request(method, url)
            .header("Authorization", format!("Bearer {}", self.api_key));

        request
    }

    pub async fn post<T: serde::de::DeserializeOwned>(
        &self,
        body: &impl serde::ser::Serialize,
    ) -> Result<T, Error> {
        let request: RequestBuilder = self.build_request(Method::POST).await;
        let request = request.json(body);
        let response = request.send().await?;
        let status = response.status();

        println!("response: {:?}", response);

        if status.is_success() {
            let parsed: T = response.json().await?;
            Ok(parsed)
        } else {
            let error_message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!("ここでエラー：{}", error_message))
        }
    }
}
