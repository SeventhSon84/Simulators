// src/plugin.rs

use plugin_interface::interface_for_plugin::Plugin;
use plugin_interface::interface_for_server::CommunicationInterface;

use serde_json::Value;
use tokio_tungstenite::tungstenite::protocol::Message;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, PartialEq)]
enum DeviceStatus {
    Armed,
    Disabled,
    Error,
}

#[derive(Clone)]
pub struct BarcodePlugin {
    status: Arc<Mutex<DeviceStatus>>,
    numeric_value: Arc<Mutex<String>>,
}

#[async_trait::async_trait]
impl Plugin for BarcodePlugin {

    fn new() -> Self {
        BarcodePlugin {
            status: Arc::new(Mutex::new(DeviceStatus::Disabled)),
            numeric_value: Arc::new(Mutex::new(String::new())),
        }
    }

    async fn handle_js_message<I: CommunicationInterface>(&self, interface: &I, text: String) {
        let json: Value = serde_json::from_str(&text).expect("Invalid JSON");

        if let Some(action) = json.get("action").and_then(|v| v.as_str()) {
            let mut status = self.status.lock().await;

            match action {
                "read" => {
                    if let Some(value) = json.get("value").and_then(|v| v.as_str()) {
                        let mut numeric_value = self.numeric_value.lock().await;
                        *numeric_value = value.to_string();
                        let read_msg = serde_json::json!({ "event": "read", "value": value });

                        interface.send_to_external(Message::Text(read_msg.to_string())).await;
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
                _ => (),
            }
        }
    }

}
impl BarcodePlugin{
    fn status_to_str(&self, status: &DeviceStatus) -> &'static str {
        match status {
            DeviceStatus::Armed => "ARMED",
            DeviceStatus::Disabled => "DISABLED",
            DeviceStatus::Error => "ERROR",
        }
    }
}
