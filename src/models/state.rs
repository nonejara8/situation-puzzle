use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum State {
    Idle,    // 開始前
    Playing, // ゲーム中
    Waiting, // 待機中
}
