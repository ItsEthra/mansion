use mansion::{server::{MansionServer, Event}, SimpleAdapter};
use serde::{Deserialize, Serialize};
use std::error::Error;


#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Message {
    AddRequest(i32, i32, String),
    AddResponse(i32, String),
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
            match e.event() {
                Event::Connected => todo!(),
                Event::Disconnected => todo!(),
                Event::Message(m) => match m {
                    Message::AddRequest(a, b, name) => {
                        let sum = *a + *b;
                        let fmt = format!("Hello, {name}");
                        e.reply(Message::AddResponse(sum, fmt)).await?
                    },
                    _ => {},
                },
            }
        }
    }

    Ok(())
}
