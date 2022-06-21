use mansion::server::MansionServer;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Message {
    AddRequest(i32, i32),
    AddResponse(i32),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    MansionServer::<Message, _>::builder(move |ctx, m| async move {
        match m {
            Message::AddRequest(a, b) => {
                if a == 0 && b == 3 {
                    ctx.close().await;
                    Ok(None)
                } else {
                    Ok(Some(Message::AddResponse(a + b)))
                }
            },
            _ => unreachable!(),
        }
    })
    .finish()
    .listen("0.0.0.0:9999")
    .await?;

    Ok(())
}
