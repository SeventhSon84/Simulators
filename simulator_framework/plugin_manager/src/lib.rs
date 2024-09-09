// src/plugin_manager.rs
use plugin_interface::interface_for_server::CommunicationInterface;
use plugin_interface::interface_for_plugin::Plugin;

use std::sync::Arc;

#[derive(Clone)]
pub struct PluginManager<I: CommunicationInterface, P: Plugin> {
    plugin: Arc<P>,
    communication_interface: Arc<I>,
}

impl<I: CommunicationInterface, P: Plugin> PluginManager<I, P> {
    pub fn new(communication_interface: Arc<I>) -> Self {
        let plugin = P::new();

        PluginManager {
            plugin: Arc::new(plugin),
            communication_interface,
        }
    }

    pub async fn handle_js_message(&self, message: String) {
        self.plugin.handle_js_message(&*self.communication_interface, message).await;
    }

    pub async fn handle_external_message(&self, message: String) {
        self.plugin.handle_external_message(&*self.communication_interface, message).await;
    }
}
