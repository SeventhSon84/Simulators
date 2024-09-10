
pub mod interface_for_server
{
    use tokio_tungstenite::tungstenite::protocol::Message;

    // Add Send + Sync bounds to the trait definition

    pub trait CommunicationInterface{
        fn send_to_js_clients(&self, message: Message);
        fn send_to_external(&self, message: Message);
    }
}

pub mod interface_for_plugin
{
    use crate::interface_for_server::CommunicationInterface;

    pub trait Plugin {
        fn new() -> Self; // Add new method to the trait
        fn handle_js_message<I: CommunicationInterface>(&mut self, interface: &I, text: String);
        fn handle_external_message<I: CommunicationInterface>(&mut self, interface: &I, text: String);
    }

}