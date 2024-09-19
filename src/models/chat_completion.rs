use serde::{Deserialize, Serialize};

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
