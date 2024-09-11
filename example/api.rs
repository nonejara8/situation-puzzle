/// no use
async fn build_request(&self, method: Method) -> reqwest::RequestBuilder {
    let url = format!("{}/chat/completions", self.api_base);
    let client = Client::new();

    let request = client
        .request(method, url)
        .header("Authorization", format!("Bearer {}", self.api_key));

    request
}

/// no use
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
