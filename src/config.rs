use std::env;

#[derive(Clone)]
pub struct Config {
    pub bot_token: String,
    pub app_token: String,
    pub default_channel: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let bot_token =
            env::var("SLACK_BOT_TOKEN").map_err(|_| "Missing required env var: SLACK_BOT_TOKEN")?;

        let app_token =
            env::var("SLACK_APP_TOKEN").map_err(|_| "Missing required env var: SLACK_APP_TOKEN")?;

        let default_channel =
            env::var("DEFAULT_CHANNEL").map_err(|_| "Missing required env var: DEFAULT_CHANNEL")?;

        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        Ok(Config {
            bot_token,
            app_token,
            default_channel,
            port,
        })
    }
}
