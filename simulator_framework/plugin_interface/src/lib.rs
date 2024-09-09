pub mod interface_for_server{
    
    use async_trait::async_trait;
    use tokio_tungstenite::tungstenite::protocol::Message;

    // Add Send + Sync bounds to the trait definition
    #[async_trait]
    pub trait CommunicationInterface: Send + Sync + 'static {
        async fn send_to_js_clients(&self, message: Message);
        async fn send_to_external(&self, message: Message);
    }
}

pub mod interface_for_plugin{

    use crate::interface_for_server::CommunicationInterface;

    use async_trait::async_trait;

    #[async_trait]
    pub trait Plugin: Send + Sync {
        fn new() -> Self; // Add new method to the trait
        async fn handle_js_message<I: CommunicationInterface>(&self, interface: &I, text: String);
        async fn handle_external_message<I: CommunicationInterface>(&self, interface: &I, text: String);
    }

}