use serde::{Deserialize, Serialize};
use mansion::server::MansionServer;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Message {
    AddRequest(i32, i32),
    AddResponse(i32),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    MansionServer::<Message, _>::builder(move |_, m| async move {
        match m {
            Message::AddRequest(a, b) => Ok(Some(Message::AddResponse(a + b))),
            _ => unreachable!(),
        }
    })
    .finish()
    .listen("0.0.0.0:9999")
    .await?;

    Ok(())
}
