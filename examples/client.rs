use serde::{Deserialize, Serialize};
use mansion::client::MansionClient;
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

    println!("Pre");
    dbg!(cl.send(Message::AddRequest(5, 10)).await?);
    println!("Post");

    Ok(())
}
