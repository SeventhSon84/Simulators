// src/plugin_manager.rs
use simulator_framework::Plugin; // Adjust if needed
use std::sync::Arc;

#[derive(Clone)]
pub struct PluginManager<I: simulator_framework::communication::CommunicationInterface> {
    plugin: Arc<Plugin>,
    communication_interface: Arc<I>,
}

impl<I: simulator_framework::communication::CommunicationInterface> PluginManager<I> {
    pub fn new(communication_interface: Arc<I>) -> Self {
        let plugin = Plugin::new(); // Adjust if needed

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
