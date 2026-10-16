use serde::{Deserialize, de::DeserializeOwned};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SlackApiError {
    #[error("Slack API error: {0}")]
    Api(String),

    #[error("Malformed Response from Slack API")]
    MalformedResponse,

    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Deserialize, Debug)]
pub struct SlackEnvelope<T> {
    ok: bool,
    error: Option<String>,
    #[serde(flatten)]
    // this takes any field that isnt ok or error, and puts it into the data field
    data: Option<T>,
}

impl<T> SlackEnvelope<T> {
    pub fn into_result(self) -> Result<T, SlackApiError> {
        if !self.ok {
            return Err(SlackApiError::Api(
                self.error.unwrap_or_else(|| "unknown_error".to_string()),
            ));
        }
        self.data.ok_or(SlackApiError::MalformedResponse)
    }
}

pub async fn parse_slack_response<T: DeserializeOwned>(
    res: reqwest::Response,
) -> Result<T, SlackApiError> {
    let envelope = res.json::<SlackEnvelope<T>>().await?;
    envelope.into_result()
}
