mod config;
mod handlers;
mod models;
mod slack;

use config::Config;

use crate::slack::{client::SlackClient, runner::Runner};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = Config::from_env().map_err(|e| anyhow::anyhow!(e))?;

    let client = SlackClient::new(&config.bot_token, &config.app_token);
    let res = client
        .post_message(
            &config.default_channel,
            "Bot is online.",
        )
        .await?;

    let (sender, mut reciever) = tokio::sync::mpsc::channel(100);
    let runner = Runner::new(client, sender);

    tokio::spawn(async move {
        if let Err(e) = runner.run().await {
            println!("Runner failed: {}", e);
        }
    });

    while let Some(event) =  reciever.recv().await {
        println!("Event Recieved Internally: {:?}", event);     
    }

    Ok(())
}
