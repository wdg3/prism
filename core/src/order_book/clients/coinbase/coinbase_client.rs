use tokio_tungstenite::tungstenite::Message;

use crate::order_book::clients::client::WebSocketClient;

use super::coinbase_adapter::CoinbaseAdapter;

pub struct CoinbaseReceiveClient {
    adapter: CoinbaseAdapter,
    client: WebSocketClient,
}

impl CoinbaseReceiveClient {
    pub async fn new() -> Self {
        return CoinbaseReceiveClient {
            adapter: CoinbaseAdapter::new().await,
            client: WebSocketClient::new("wss://ws-feed.exchange.coinbase.com".to_string()).await
        }
    }

    pub async fn init(&mut self) {
        let sub_message: String = "{\"type\":\"subscribe\",\"product_ids\":[\"ETH-USD\"],\"channels\":[\"ticker\"]}".to_string();
        self.client.send(Message::Text(sub_message)).await;
        self.client.receive().await;
    }
}

pub struct CoinbaseSendClient {
    client: WebSocketClient,
}

impl CoinbaseSendClient {
    pub async fn new() -> Self {
        return CoinbaseSendClient {
            client: WebSocketClient::new("wss://ws-feed.exchange.coinbase.com".to_string()).await
        }
    }
}