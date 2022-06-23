use mansion::{client::MansionClient, EncryptionTarget, EncryptedClientAdapter};
use rsa::{RsaPublicKey, pkcs1::FromRsaPublicKey};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Message {
    EncryptionRequest,
    EncryptionResponse,
    AddRequest(i32, i32, String),
    AddResponse(i32, String),
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
    // env_logger::builder().filter_level(log::LevelFilter::Trace).init();

    let client = MansionClient::<Message>::builder()
        .with_adapter(EncryptedClientAdapter::new(
            RsaPublicKey::from_pkcs1_pem(include_str!("../public.pem")).unwrap(),
        ))
        .connect("127.0.0.1:9999")
        .await
        .unwrap();

    println!("Encrypt start");
    dbg!(client.send_wait(Message::EncryptionRequest).await?);
    println!("Encrypt end");

    println!("Pre");
    dbg!(client.send_wait(Message::AddRequest(1, 2, "Yohan".into())).await?);
    println!("Post");

    Ok(())
}
