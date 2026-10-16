use crate::slack::http::SlackApiError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SocketError {
    #[error("WebSocket error: {0}")]
    Tungstenite(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API request failed: {0}")]
    Http(#[from] SlackApiError),

    #[error("Event channel closed {0}")]
    ChannelClosed(#[from] tokio::sync::mpsc::error::SendError<IncomingEvent>),

    #[error("Envelope missing ID")]
    MissingEnvelopeId,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IncomingEvent {
    Hello {
      connection_info: serde_json::Value,
      num_connections: u8,
    },
    Disconnect {
        reason: Option<String>,
    },
    EventsApi {
        envelope_id: String,
        payload: serde_json::Value,
        #[serde(default)]
        accepts_response_payload: bool,
    },
    Interactive {
        envelope_id: String,
        payload: serde_json::Value,
        #[serde(default)]
        accepts_response_payload: bool,
    },
    SlashCommands {
        envelope_id: String,
        payload: serde_json::Value,
        #[serde(default)]
        accepts_response_payload: bool,
    },
    #[serde(other)]
    Unknown,
}

impl IncomingEvent {
    pub fn envelope_id(&self) -> Option<&str> {
        match self {
            IncomingEvent::EventsApi { envelope_id, .. }
            | IncomingEvent::Interactive { envelope_id, .. }
            | IncomingEvent::SlashCommands { envelope_id, .. } => Some(envelope_id.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AcknowledgeEvent {
    pub envelope_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

impl AcknowledgeEvent {
    pub fn new(envelope_id: String, payload: Option<serde_json::Value>) -> Self {
        AcknowledgeEvent {
            envelope_id,
            payload,
        }
    }

    pub fn to_json(&self) -> Result<String, SocketError> {
        let json_str = serde_json::to_string(self)?;
        Ok(json_str)
    }
}

