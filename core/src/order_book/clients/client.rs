use futures_util::{StreamExt, SinkExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Error;
use tokio_tungstenite::{ MaybeTlsStream, WebSocketStream, connect_async, connect_async_with_config};
use tokio_tungstenite::tungstenite::protocol::{Message, WebSocketConfig};

pub struct FIXClient {

}

impl FIXClient {

}

pub struct UDPClient {

}

impl UDPClient {

}

pub struct WebSocketClient {
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WebSocketClient {
    pub async fn new(address: String) -> Self {
        let config = WebSocketConfig {
            max_send_queue: None,
            max_frame_size: None,
            max_message_size: None,
            accept_unmasked_frames: true,
        };
        let result = connect_async_with_config(
            address,
            Some(config),
            true
        ).await;
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

    pub async fn receive(&mut self) -> Option<Result<Message, Error>> {
        return self.ws_stream.next().await;
    }
}