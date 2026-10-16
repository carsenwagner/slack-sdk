use serde::Deserialize;

#[derive(Deserialize)]
pub struct MessagePostResult {
    pub channel: String,
    pub ts: String,
}
