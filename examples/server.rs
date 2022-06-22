use mansion::server::MansionServer;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, net::SocketAddr};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Message {
    AddRequest(i32, i32),
    AddResponse(i32),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    MansionServer::<Message, Mutex<HashMap<SocketAddr, i32>>, _, _>::builder(
        move |ctx, m| async move {
            match m {
                Message::AddRequest(a, b) => {
                    if a == 0 && b == 3 {
                        ctx.close().await;
                        Ok(None)
                    } else {
                        let lock = &mut *ctx.state().lock().await;
                        let t: &mut i32 = lock.entry(ctx.addr()).or_default();
                        *t += 1;
                        dbg!(ctx.addr(), *t, lock.len());

                        Ok(Some(Message::AddResponse(a + b)))
                    }
                }
                _ => unreachable!(),
            }
        },
    )
    // This is required unfortunatelly, even if you don't want to use it.
    .on_disconnect(move |st, addr| async move {
        st.lock().await.remove(&addr);

        Ok(())
    })
    .state(Mutex::new(HashMap::new()))
    .finish()
    .listen("0.0.0.0:9999")
    .await?;

    Ok(())
}
