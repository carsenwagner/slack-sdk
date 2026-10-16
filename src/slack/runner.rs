use tokio::sync::mpsc;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use crate::slack::{client::SlackClient, socket::{AcknowledgeEvent, IncomingEvent, SocketError}};


pub struct Runner {
   pub client: SlackClient, 
   pub tx: mpsc::Sender<IncomingEvent>,
}

impl Runner {
    pub fn new(client: SlackClient, tx: mpsc::Sender<IncomingEvent>) -> Self {
        Runner {
            client,
            tx,
        }
    }


    pub async fn run(&self) -> Result<(), SocketError> {
        
        loop {
            let result = self.run_session().await;
            match result {
                Ok(()) => println!("Reconnecting"),
                Err(err) => println!("{}", err),
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }  
    } 

    pub async fn run_session(&self) -> Result<(), SocketError> {
        let url = self.client.open_socket_url().await?;
        
        let (mut ws_stream, _res) = tokio_tungstenite::connect_async(url).await?;

        while let Some(res) = ws_stream.next().await { 
            match res? {
                Message::Text(t) => {
                    let event = serde_json::from_str::<IncomingEvent>(t.as_str())?;
                    if let Some(id) = event.envelope_id() { 
                        let ack =  AcknowledgeEvent::new(id.to_string(), None);
                        let ack_json = ack.to_json()?;
                        let message = Message::Text(ack_json);
                        println!("{:?}", &message);
                        ws_stream.send(message).await?;
                    } 
                    match event {
                        IncomingEvent::Hello { .. } => println!("Hello Recieved"),
                        IncomingEvent::Disconnect { .. } =>  return Ok(()),
                        IncomingEvent::Unknown => println!("Unknown Event {}", t),
                        IncomingEvent::EventsApi { .. }
                        | IncomingEvent::Interactive { .. }
                        | IncomingEvent::SlashCommands { .. } => {
                            self.tx.send(event).await?;
                        }
                    }; 
                },
                Message::Close => println!("Closed"),
                _ => {},
            }
        }  
        Ok(())
    }
}
