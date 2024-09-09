// src/plugin.rs

use plugin_interface::interface_for_plugin::Plugin;
use plugin_interface::interface_for_server::CommunicationInterface;

pub struct DefaultPlugin;

#[async_trait::async_trait]
impl Plugin for DefaultPlugin {
    fn new() -> Self {
        DefaultPlugin
    }
    async fn handle_js_message<I: CommunicationInterface>(&self, _interface: &I, _text: String)
    {} 
    async fn handle_external_message<I: CommunicationInterface>(&self, _interface: &I, _text: String) 
    {}

}