use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

pub struct WebSocketClient {
    pub address: String,
}

impl WebSocketClient {
    pub async fn init(&self) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
        let (ws_stream, _) = connect_async(&self.address).await.expect("Failed to connect");

        return ws_stream
    }
}