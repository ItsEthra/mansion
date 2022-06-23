use mansion::{server::{MansionServer, Event}, EncryptedServerAdapter, EncryptionTarget};
use rsa::{RsaPrivateKey, pkcs8::FromPrivateKey};
use serde::{Deserialize, Serialize};
use std::error::Error;
use log::LevelFilter;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Message {
    EncryptionRequest,
    EncryptionResponse,
    AddRequest(i32, i32),
    AddResponse(i32),
}

impl EncryptionTarget for Message {
    fn request() -> Self {
        Self::EncryptionRequest
    }

    fn response() -> Self {
        Self::EncryptionResponse
    }

    fn is_request(&self) -> bool {
        matches!(self, Self::EncryptionRequest)
    }

    fn is_response(&self) -> bool {
        matches!(self, Self::EncryptionResponse)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().filter_level(LevelFilter::Trace).init();
    
    let server = MansionServer::<Message>::builder()
        .with_adapter(EncryptedServerAdapter::new(
            RsaPrivateKey::from_pkcs8_pem(include_str!("../private.pem")).unwrap(),
        ))
        .listen("0.0.0.0:9999")
        .await?;

    while let Some(e) = server.message().await {
        dbg!(&e);
        
        match e.event() {
            Event::Message(m) => match m {
                Message::EncryptionRequest => e.reply(Message::EncryptionResponse).await?,
                Message::AddRequest(a, b) => {
                    let sum = *a + *b;
                    e.reply(Message::AddResponse(sum)).await?
                },
                _ => {}
            },
            _ => {}
        }
    }

    Ok(())
}
