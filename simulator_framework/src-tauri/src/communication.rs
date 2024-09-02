use async_trait::async_trait;
use tokio_tungstenite::tungstenite::protocol::Message;

#[async_trait]
pub trait CommunicationInterface{
    async fn send_to_js_clients(&self, message: Message);
    async fn send_to_external(&self, message: Message);
}
