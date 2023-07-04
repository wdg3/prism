use futures_util::{StreamExt, SinkExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{ MaybeTlsStream, WebSocketStream, connect_async};
use tokio_tungstenite::tungstenite::protocol::Message;

pub struct WebSocketClient {
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WebSocketClient {
    pub async fn new(address: String) -> Self {
        let result = connect_async(address).await;
        match result {
            Ok((ws_stream, _)) => return WebSocketClient {ws_stream},
            Err(result) => {
                println!("Error connecting: {:?}", result.to_string());
                panic!();
            }
        };
    }

    pub async fn send(&mut self, msg: Message) {
        if let Err(result) = &self.ws_stream.send(msg).await {
            println!("Error sending message: {:?}", result);
        }
    }

    pub async fn receive(&mut self) {
        while let Some(msg) = &self.ws_stream.next().await {
            println!("{:?}", msg);
        }
    }
}