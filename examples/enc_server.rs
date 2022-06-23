use mansion::{server::MansionServer, SimpleAdapter};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Message {
    EncryptionRequest,
    EncryptionResponse,
    AddRequest(i32, i32),
    AddResponse(i32),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = MansionServer::<Message>::builder()
        .with_adapter(SimpleAdapter::default())
        .listen("0.0.0.0:9999")
        .await?;

    while let Some(e) = server.message().await {
        dbg!(&e);

        if e.is_message() {
            e.reply(Message::AddResponse(5)).await?;
        }
    }

    Ok(())
}
