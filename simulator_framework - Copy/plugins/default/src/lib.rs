// src/plugin.rs

use plugin_interface::interface_for_plugin::Plugin;
use plugin_interface::interface_for_server::CommunicationInterface;

pub struct DefaultPlugin;
 

impl Plugin for DefaultPlugin 
{
    fn new() -> Self 
    {
        DefaultPlugin
    }
    fn handle_js_message<I: CommunicationInterface>(&mut self, _interface: &I, _text: String)
    {} 

    fn handle_external_message<I: CommunicationInterface>(&mut self, _interface: &I, _text: String) 
    {}

}
    