// src/plugin.rs

use plugin_interface::interface_for_plugin::Plugin;
use plugin_interface::interface_for_server::CommunicationInterface;

use serde_json::Value;
use tokio_tungstenite::tungstenite::protocol::Message;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, PartialEq)]
enum DeviceStatus {
    Armed,
    Disabled,
    Error,
}



#[derive(Clone)]
pub struct BNAPlugin {
    status: Arc<Mutex<DeviceStatus>>,
    numeric_value: Arc<Mutex<String>>,
    read_state: Arc<Mutex<bool>>,
}

impl BNAPlugin{
    fn status_to_str(&self, status: &DeviceStatus) -> &'static str {
        match status {
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
            status: Arc::new(Mutex::new(DeviceStatus::Disabled)),
            numeric_value: Arc::new(Mutex::new(String::new())),
            read_state: Arc::new(Mutex::new(false))
        }
    }

    async fn handle_js_message<I: CommunicationInterface>(&self, interface: &I, text: String) {
        let json: Value = serde_json::from_str(&text).expect("Invalid JSON");

        if let Some(action) = json.get("action").and_then(|v| v.as_str()) {
            let mut status = self.status.lock().await;

            match action {
                "read" => {

                    if let Some(value) = json.get("value").and_then(|v| v.as_str()) 
                    {
                        *status = DeviceStatus::Disabled;
                        
                        // Lock the read_state mutex and update its value
                        let mut read_state = self.read_state.lock().await;
                        *read_state = true;

                        let mut numeric_value = self.numeric_value.lock().await;
                        *numeric_value = value.to_string();

                        let read_msg = serde_json::json!({ "event": "read", "value": *numeric_value });

                        interface.send_to_external(Message::Text(read_msg.to_string())).await;

                        let status_msg = serde_json::json!({ "event": "statusChange", "status": self.status_to_str(&status) });
                        interface.send_to_js_clients(Message::Text(status_msg.to_string())).await;
                        interface.send_to_external(Message::Text(status_msg.to_string())).await;
                    }
                }
                "error" => {
                    if *status == DeviceStatus::Error {
                        *status = DeviceStatus::Disabled;
                    } else {
                        *status = DeviceStatus::Error;
                    }
                    
                    let status_msg = serde_json::json!({ "event": "statusChange", "status": self.status_to_str(&status) });

                    interface.send_to_js_clients(Message::Text(status_msg.to_string())).await;
                    interface.send_to_external(Message::Text(status_msg.to_string())).await;
                }
                _ => (),
            }
        }
    }

    async fn handle_external_message<I: CommunicationInterface>(&self, interface: &I, text: String) {
        let json: Value = serde_json::from_str(&text).expect("Invalid JSON");

        if let Some(action) = json.get("action").and_then(|v| v.as_str()) {
            let mut status = self.status.lock().await;

            if *status == DeviceStatus::Error && (action == "enable" || action == "disable") {
                return;
            }

            match action {
                "enable" => {
                    *status = DeviceStatus::Armed;
                    let status_msg = serde_json::json!({ "event": "statusChange", "status": "ARMED" });

                    interface.send_to_js_clients(Message::Text(status_msg.to_string())).await;
                    interface.send_to_external(Message::Text(status_msg.to_string())).await;
                }
                "disable" => {
                    *status = DeviceStatus::Disabled;
                    let status_msg = serde_json::json!({ "event": "statusChange", "status": "DISABLED" });

                    interface.send_to_js_clients(Message::Text(status_msg.to_string())).await;
                    interface.send_to_external(Message::Text(status_msg.to_string())).await;
                }
                "query_status" =>{
                    let status_msg = serde_json::json!({ "event": "statusChange", "status": self.status_to_str(&*status) });
                    interface.send_to_external(Message::Text(status_msg.to_string())).await;
                }
                "confirm_read" =>{
                    let status_msg = serde_json::json!({ "event": "confirm_read" });
                    interface.send_to_external(Message::Text(status_msg.to_string())).await;
                }
                _ => (),
            }
        }
    }

}

