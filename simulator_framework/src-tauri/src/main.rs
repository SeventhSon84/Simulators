#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//pub mod communication;

use simulator_framework::communication::CommunicationInterface; // Adjust the path as needed

use tauri::Manager;
use tauri::command;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc::{self, UnboundedSender}};
use tokio::sync::broadcast;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

mod plugin_manager; // Include plugin manager module

use plugin_manager::PluginManager;

#[derive(Deserialize)]
struct Config {
    js_port: u16,
    external_port: u16,
}

fn load_config() -> Config {
    let exe_path = std::env::current_exe().expect("Failed to get current executable path");
    let config_path = exe_path.parent().unwrap().join("config.json");
    let mut file = File::open(&config_path).expect("Unable to open config file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read config file");
    serde_json::from_str(&contents).expect("Invalid config file format")
}

#[command]
fn get_js_port() -> u16 {
    let config = load_config();
    config.js_port
}

// Define the `AppState` without `PluginManager`
#[derive(Clone)]
struct AppState {
    external_client_tx: Arc<Mutex<Option<UnboundedSender<Message>>>>,  // Sender for external client
    js_clients_tx: Arc<Mutex<broadcast::Sender<Message>>>, // Broadcast sender for JS clients
}

#[async_trait::async_trait]
impl CommunicationInterface for AppState {
    async fn send_to_js_clients(&self, message: Message) {
        if let Some(sender) = &*self.external_client_tx.lock().await {
            let _ = sender.send(message);
        }
    }

    async fn send_to_external(&self, message: Message) {
        let _ = self.js_clients_tx.lock().await.send(message);
    }
}

#[tokio::main]
async fn main() {


    tauri::Builder::default()
        .setup(move |app| {
            let (js_clients_tx, _) = broadcast::channel(16);

            let state = Arc::new(AppState {
                js_clients_tx: Arc::new(Mutex::new(js_clients_tx)),
                external_client_tx: Arc::new(Mutex::new(None)),
            });
            
            // Create the PluginManager with a reference to `state`
            let plugin_manager = Arc::new(PluginManager::new(state.clone()));
            
            let config = load_config();

            tauri::async_runtime::spawn(start_js_websocket_server(state.clone(), plugin_manager.clone(), config.js_port));
            tauri::async_runtime::spawn(start_external_websocket_server(state.clone(), plugin_manager.clone(), config.external_port));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_js_port])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn start_js_websocket_server<I: CommunicationInterface>(state: Arc<AppState>, plugin_manager: Arc<PluginManager<I>>, port: u16) 
where
    I: CommunicationInterface
{

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to bind JS WebSocket server");
    println!("JS WebSocket server running on ws://127.0.0.1:{}", port);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_js_client(state.clone(), plugin_manager.clone(), stream));
    }
}


async fn start_external_websocket_server<I: CommunicationInterface >(state: Arc<AppState>, plugin_manager: Arc<PluginManager<I>>,port: u16)
where
    I: CommunicationInterface 
{
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to bind External WebSocket server");
    println!("External WebSocket server running on ws://127.0.0.1:{}", port);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_external_client(state.clone(), plugin_manager.clone(), stream));
    }
}


async fn handle_js_client<I: CommunicationInterface>(state: Arc<AppState>, plugin_manager: Arc<PluginManager<I>>, stream: tokio::net::TcpStream) {
    let ws_stream = accept_async(stream).await.expect("Error during WebSocket handshake");
    let (mut write, mut read) = ws_stream.split();
    let mut rx = state.js_clients_tx.lock().await.subscribe();

    // Forward broadcast messages to this client
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if write.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = read.next().await {
        if let Message::Text(text) = msg {
            // Forward the message to the plugin manager for handling
            plugin_manager.handle_js_message(text).await;
        }
    }
}

async fn handle_external_client<I: CommunicationInterface >(state: Arc<AppState>, plugin_manager: Arc<PluginManager<I>>, stream: tokio::net::TcpStream) {
    
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            println!("Error during WebSocket handshake: {}", e);
            return; // Exit the function if the handshake fails
        }
    };
    
    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Set the sender for the external client
    *state.external_client_tx.lock().await = Some(tx);

    // Relay messages from the state to the external client
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if write.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = read.next().await {
        if let Message::Text(text) = msg {
            // Forward the message to the plugin manager for handling
            plugin_manager.handle_external_message(text).await;
        }
    }
}