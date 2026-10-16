use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    models::MessagePostResult,
    slack::http::{SlackApiError, parse_slack_response},
};

#[derive(Serialize)]
struct PostMessageRequest<'a> {
    channel: &'a str,
    text: &'a str,
}

#[derive(Deserialize)]
pub struct ConnectionsOpenResponse {
    url: String,
}

#[derive(Clone, Debug)]
pub struct SlackClient {
    client: Client,
    bot_token: String,
    base_url: String,
    app_token: String,
}

impl SlackClient {
    pub fn new(bot_token: impl Into<String>, app_token: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            bot_token: bot_token.into(),
            base_url: "https://slack.com/api".to_string(),
            app_token: app_token.into(),
        }
    }

    pub async fn open_socket_url(&self) -> Result<String, SlackApiError> {
        let response = self
            .client
            .post(format!("{}/apps.connections.open", &self.base_url))
            .bearer_auth(&self.app_token)
            .send()
            .await?;
 
        let body: ConnectionsOpenResponse = parse_slack_response(response).await?;
        Ok(body.url)
    }

    pub async fn post_message(
        &self,
        channel: &str,
        text: &str,
    ) -> Result<MessagePostResult, SlackApiError> {
        let payload = PostMessageRequest { channel, text };

        let url = format!("{}/chat.postMessage", &self.base_url);

        let response = self
            .client
            .post(url)
            .bearer_auth(&self.bot_token)
            .json(&payload)
            .send()
            .await?;

        parse_slack_response(response).await
    }
}
