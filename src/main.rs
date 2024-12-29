use std::sync::Arc;

use rocket::{
    futures::{SinkExt, StreamExt},
    tokio::{
        self,
        io::{AsyncReadExt, AsyncWriteExt},
    },
};
use rocket_ws::Message;
use russh::{client, keys::ssh_key};

#[macro_use]
extern crate rocket;

struct Client;

#[async_trait]
impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

#[get("/ssh/<host>?<user>&<password>&<dimension>")]
async fn connect_ssh(
    host: &str,
    user: &str,
    password: &str,
    dimension: Option<(u32, u32)>,
    ws: rocket_ws::WebSocket,
) -> rocket_ws::Channel<'static> {
    let mut session = russh::client::connect(Arc::new(client::Config::default()), host, Client {})
        .await
        .unwrap();

    session.authenticate_password(user, password).await.unwrap();

    let channel = session.channel_open_session().await.unwrap();

    let (columns, rows) = dimension.unwrap_or((80, 20));
    channel
        .request_pty(true, "xterm", columns, rows, 0, 0, &[])
        .await
        .unwrap();
    channel.request_shell(true).await.unwrap();

    ws.channel(move |mut stream| {
        Box::pin(async move {
            let mut ssh_stream = channel.into_stream();
            let mut buf = Vec::new();

            loop {
                tokio::select! {
                    res = ssh_stream.read_buf(&mut buf) => {
                        match res {
                            Ok(_) => {
                                stream.send(Message::Binary(buf.clone())).await.unwrap();
                                // print!("{}", String::from_utf8_lossy(&buf));
                                // std::io::stdout().flush().unwrap();
                                buf = Vec::new();
                            }, Err(_) => {
                                return Ok(());
                            }
                        }
                    },
                    Some(Ok(message)) = stream.next() => {
                        if message.is_close() {
                            break;
                        }

                        ssh_stream.write(message.to_text().unwrap().as_bytes()).await.unwrap();
                    }
                }
            }

            Ok(())
        })
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![connect_ssh])
}
