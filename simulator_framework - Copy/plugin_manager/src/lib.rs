// src/plugin_manager.rs
use plugin_interface::interface_for_server::CommunicationInterface;
use plugin_interface::interface_for_plugin::Plugin;

use std::sync::{Arc, Mutex};

pub struct PluginManager<I: CommunicationInterface, P: Plugin> {
    plugin: Mutex<P>,
    communication_interface: Arc<I>,
}

impl<I: CommunicationInterface, P: Plugin> PluginManager<I, P> {
    pub fn new(communication_interface: Arc<I>) -> Self {
        let plugin = P::new();

        PluginManager {
            plugin: Mutex::new(plugin),
            communication_interface,
        }
    }

    pub fn handle_js_message(&self, message: String) 
    {
        // Your synchronous code here
        let mut plugin = self.plugin.lock().unwrap();
        plugin.handle_js_message(&*self.communication_interface, message);
    }

    pub fn handle_external_message(&self, message: String) 
    {
        // Your synchronous code here
        let mut plugin = self.plugin.lock().unwrap();
        plugin.handle_external_message(&*self.communication_interface, message);
    }
    
}
