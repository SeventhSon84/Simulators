#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use plugin_interface::interface_for_server::CommunicationInterface;
use plugin_interface::interface_for_plugin::Plugin;

use tauri::command;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};

use std::sync::Arc;
use tokio::sync::{Mutex, mpsc::{self, UnboundedSender}};

use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use tokio::task::JoinHandle;

use plugin_manager::PluginManager;

#[cfg(feature = "feature-barcode")]
use barcode_plugin::BarcodePlugin; // or another plugin

#[cfg(feature = "feature-barcode")]
type SelectedPlugin = BarcodePlugin;

#[cfg(feature = "feature-bna")]
use bna_plugin::BNAPlugin; // or another plugin

#[cfg(feature = "feature-bna")]
type SelectedPlugin = BNAPlugin;

#[cfg(feature = "feature-card")]
use card_plugin::CardPlugin;
#[cfg(feature = "feature-card")]
type SelectedPlugin = CardPlugin;


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
    js_clients_tx: Arc<Mutex<Option<UnboundedSender<Message>>>>, // Broadcast sender for JS clients
}
impl AppState{

    fn send_to_client(&self, message: Message, client_channel: &Arc<Mutex<Option<UnboundedSender<Message>>>>)
    {
        // This function is called synchronously, so we use `block_in_place`
        // to run async code in a blocking context.

        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let client_channel_mut = client_channel.lock().await;
                if let Some(sender) = &*client_channel_mut {
                    let _ = sender.send(message);
                }
            });
        });
    }
}

impl CommunicationInterface for AppState 
{
    fn send_to_js_clients(&self, message: Message) 
    {
        self.send_to_client(message, &self.js_clients_tx);
    }

    fn send_to_external(&self, message: Message) 
    {
        self.send_to_client(message, &self.external_client_tx);
    }
}


#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(move |_app| {

            let state = Arc::new(AppState {
                js_clients_tx: Arc::new(Mutex::new(None)),
                external_client_tx: Arc::new(Mutex::new(None)),
            });

            let plugin_manager = Arc::new(Mutex::new(PluginManager::<AppState, SelectedPlugin>::new(state.clone())));

            let config = load_config();

            // Spawning the WebSocket server with JS client handler
            tauri::async_runtime::spawn(start_websocket_server(
                config.js_port,
                {
                    let state = state.clone();
                    let plugin_manager = plugin_manager.clone();
                    // non ti preoccupare di avere tokio::spawn annidati
                    move |stream| tokio::spawn(handle_js_client(state.clone(), plugin_manager.clone(), stream))
                },
            ));
            

            // Spawning the WebSocket server with External client handler
            tauri::async_runtime::spawn(start_websocket_server(
                config.external_port,
                {
                    let state = state.clone();
                    let plugin_manager = plugin_manager.clone();
                    // non ti preoccupare di avere tokio::spawn annidati
                    move |stream| tokio::spawn(handle_external_client(state.clone(), plugin_manager.clone(), stream))
                },
            ));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_js_port])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}



// The merged WebSocket server function with simplified parameters
async fn start_websocket_server<F>(port: u16, handler: F)
where
    F: Fn(tokio::net::TcpStream) -> JoinHandle<()>,
{
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to bind WebSocket server");
    println!("WebSocket server running on ws://127.0.0.1:{}", port);

    while let Ok((stream, _)) = listener.accept().await {
        // non ti preoccupare di avere tokio::spawn annidati
        tokio::spawn(handler(stream));
    }
}

async fn handle_js_client<P: Plugin>(state: Arc<AppState>, plugin_manager: Arc<Mutex<PluginManager<AppState, P>>>, stream: tokio::net::TcpStream) {
   // ad ogni nuova connessione si finisce qui...
   if state.js_clients_tx.lock().await.is_some() {
    // Refuse connection if another external client is already connected
    println!("Connection refused: Another external client is already connected.");
    return;
    }

    // accetta la connessione...
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            println!("Error during WebSocket handshake: {}", e);
            return; // Exit the function if the handshake fails
        }
    };

    // se la connessione e' valida si prosegue da qui...

    // si splitta il canale di comunicazione con l'OP in due (write e read)
    let (mut write_to_socket, mut read_from_socket) = ws_stream.split();

    // qui si crea un nuovo canale di comunication tra questo thread e il thread che gestisce la richiesta...
    let (tx, mut rx) = mpsc::unbounded_channel();

    // salva il lato tx del canale interno nella variabile apposita...
    *state.js_clients_tx.lock().await = Some(tx);

    // qui si fa partire un altro thread che sta in ascolto per la ricezione della risposta (interna),
    // quando si riceve la risposta (generata da un altro thread) qui si manda la risposta all'OP
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await
        {
            if write_to_socket.send(msg).await.is_err() 
            {
                // errore sul socket... annulla questa sessione...
                break;
            }
        }
    });

    // qui si va a gestire la richiesta dell'OP (su questo thread...), 
    // ed eventuali future richieste da questa connessione...
    while let Some(Ok(msg)) = read_from_socket.next().await 
    {
        if let Message::Text(text) = msg 
        {
            // Forward the message to the plugin manager for handling
            // Forward the message to the plugin manager for handling
            let lock_on_plugin = plugin_manager.lock().await;
            lock_on_plugin.handle_js_message(text);
        }
    }

    println!("Connection closed");
    *state.js_clients_tx.lock().await = None;
}

async fn handle_external_client<P: Plugin>(state: Arc<AppState>, plugin_manager: Arc<Mutex<PluginManager<AppState, P>>>, stream: tokio::net::TcpStream) 
{
    // ad ogni nuova connessione si finisce qui...
    if state.external_client_tx.lock().await.is_some() {
        // Refuse connection if another external client is already connected
        println!("Connection refused: Another external client is already connected.");
        return;
    }

    // accetta la connessione...
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            println!("Error during WebSocket handshake: {}", e);
            return; // Exit the function if the handshake fails
        }
    };
    
    // se la connessione e' valida si prosegue da qui...

    // si splitta il canale di comunicazione con l'OP in due (write e read)
    let (mut write_to_socket, mut read_from_socket) = ws_stream.split();

    // qui si crea un nuovo canale di comunication tra questo thread e il thread che gestisce la richiesta...
    let (tx, mut rx) = mpsc::unbounded_channel();

    // salva il lato tx del canale interno nella variabile apposita...
    *state.external_client_tx.lock().await = Some(tx);

    // qui si fa partire un altro thread che sta in ascolto per la ricezione della risposta (interna),
    // quando si riceve la risposta (generata da un altro thread) qui si manda la risposta all'OP
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await
        {
            if write_to_socket.send(msg).await.is_err() 
            {
                // errore sul socket... annulla questa sessione...
                break;
            }
        }
    });

    // qui si va a gestire la richiesta dell'OP (su questo thread...), 
    // ed eventuali future richieste da questa connessione...
    while let Some(Ok(msg)) = read_from_socket.next().await 
    {
        if let Message::Text(text) = msg 
        {
            // Forward the message to the plugin manager for handling
              // Forward the message to the plugin manager for handling
              let lock_on_plugin = plugin_manager.lock().await;
              lock_on_plugin.handle_external_message(text);
        }
    }
    
    println!("Connection closed");
    *state.external_client_tx.lock().await = None;
}
    