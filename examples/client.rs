use mansion::client::MansionClient;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Message {
    AddRequest(i32, i32),
    AddResponse(i32),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cl = MansionClient::<Message>::builder()
        .connect("127.0.0.1:9999")
        .await?;

    dbg!(cl.send_wait(Message::AddRequest(0, 1)).await?);
    dbg!(cl.send_wait(Message::AddRequest(0, 2)).await?);
    dbg!(cl.send_wait(Message::AddRequest(0, 3)).await?);
    dbg!(cl.send_wait(Message::AddRequest(0, 4)).await?);

    Ok(())
}
