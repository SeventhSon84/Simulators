// src/plugin.rs

use plugin_interface::interface_for_plugin::Plugin;
use plugin_interface::interface_for_server::CommunicationInterface;

use serde_json::Value;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Clone, Debug, PartialEq)]
enum DeviceStatus {
    Armed,
    Disabled,
    Error,
}


#[derive(Clone)]
pub struct BNAPlugin {
    status: DeviceStatus,
    numeric_value: String,
    read_state: bool,
}

impl BNAPlugin
{
    fn status_to_str(&self, status: &DeviceStatus) -> &'static str 
    {
        match status 
        {
            DeviceStatus::Armed => "ARMED",
            DeviceStatus::Disabled => "DISABLED",
            DeviceStatus::Error => "ERROR",
        }
    }
}

#[async_trait::async_trait]
impl Plugin for BNAPlugin {

    fn new() -> Self {
        BNAPlugin {
            status: DeviceStatus::Disabled,
            numeric_value: String::new(),
            read_state: false
        }
    }

    fn handle_js_message<I: CommunicationInterface>(&mut self, interface: &I, text: String) 
    {
        let json: Value = serde_json::from_str(&text).expect("Invalid JSON");

        if let Some(action) = json.get("action").and_then(|v| v.as_str()) 
        {
            match action {
                "read" => {

                    if let Some(value) = json.get("value").and_then(|v| v.as_str()) 
                    {
                        self.status = DeviceStatus::Disabled;
                        
                        // Lock the read_state mutex and update its value
                        self.read_state = true;

                        self.numeric_value = value.to_string();

                        let read_msg = serde_json::json!({ "event": "read", "value": self.numeric_value });

                        interface.send_to_external(Message::Text(read_msg.to_string()));

                        let status_msg = serde_json::json!({ "event": "statusChange", "status": self.status_to_str(&self.status) });
                        interface.send_to_js_clients(Message::Text(status_msg.to_string()));
                        interface.send_to_external(Message::Text(status_msg.to_string()));
                    }
                }
                "error" => {
                    if self.status == DeviceStatus::Error {
                        self.status = DeviceStatus::Disabled;
                    } else {
                        self.status = DeviceStatus::Error;
                    }
                    
                    let status_msg = serde_json::json!({ "event": "statusChange", "status": self.status_to_str(&self.status) });

                    interface.send_to_js_clients(Message::Text(status_msg.to_string()));
                    interface.send_to_external(Message::Text(status_msg.to_string()));
                }
                _ => (),
            }
        }
    }

    fn handle_external_message<I: CommunicationInterface>(&mut self, interface: &I, text: String) {
        let json: Value = serde_json::from_str(&text).expect("Invalid JSON");

        if let Some(action) = json.get("action").and_then(|v| v.as_str()) {

            if self.status == DeviceStatus::Error && (action == "enable" || action == "disable") {
                return;
            }

            match action {
                "enable" => {
                    self.status = DeviceStatus::Armed;
                    let status_msg = serde_json::json!({ "event": "statusChange", "status": "ARMED" });

                    interface.send_to_js_clients(Message::Text(status_msg.to_string()));
                    interface.send_to_external(Message::Text(status_msg.to_string()));
                }
                "disable" => {
                    self.status = DeviceStatus::Disabled;
                    let status_msg = serde_json::json!({ "event": "statusChange", "status": "DISABLED" });

                    interface.send_to_js_clients(Message::Text(status_msg.to_string()));
                    interface.send_to_external(Message::Text(status_msg.to_string()));
                }
                "query_status" =>{
                    let status_msg = serde_json::json!({ "event": "statusChange", "status": self.status_to_str(&self.status) });
                    interface.send_to_external(Message::Text(status_msg.to_string()));
                }
                "confirm_read" =>{
                    let status_msg = serde_json::json!({ "event": "confirm_read" });
                    interface.send_to_external(Message::Text(status_msg.to_string()));
                }
                _ => (),
            }
        }
    }

}
