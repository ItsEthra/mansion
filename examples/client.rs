use mansion::{client::MansionClient, SimpleAdapter};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Message {
    AddRequest(i32, i32, String),
    AddResponse(i32, String),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = MansionClient::<Message>::builder()
        .with_adapter(SimpleAdapter::default())
        .connect("127.0.0.1:9999")
        .await
        .unwrap();

    println!("Pre");
    dbg!(client.send_wait(Message::AddRequest(1, 2, "Yohan".into())).await?);
    println!("Post");

    Ok(())
}
