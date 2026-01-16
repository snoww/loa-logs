use std::{net::SocketAddr, sync::Arc};
use anyhow::Result;
use axum::{extract::{ws::{Message, WebSocket}, *}, response::IntoResponse, routing::any, Router};
use log::warn;
use serde::Serialize;
use serde_json::Value;
use tokio::{sync::{mpsc, Mutex}, task::spawn};

use crate::live::packet::LoaPacket;

#[cfg(test)]
use mockall::automock;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaLogsMessage<'a> {
    version: &'a str,
    data: LoaLogsMessageType
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "data")] 
pub enum LoaLogsMessageType {
    Packet(LoaPacket),
    Snapshot(Value)
}

#[cfg_attr(test, automock)]
pub trait BroadcastManager {
    fn is_enabled(&self) -> bool;
    fn send(&self, message: LoaLogsMessageType) -> Result<()>;
}

pub struct DefaultBroadcastManager {
    tx: mpsc::UnboundedSender<LoaLogsMessageType>,
}

impl BroadcastManager for DefaultBroadcastManager {
    fn send(&self, packet: LoaLogsMessageType) -> Result<()> {
        self.tx.send(packet).map_err(|err| anyhow::anyhow!("broadcast channel closed: {}", err))
    }
    
    fn is_enabled(&self) -> bool {
        !self.tx.is_closed()
    }
}

impl DefaultBroadcastManager {
    pub fn new(version: String) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<LoaLogsMessageType>();
        let addr = option_env!("BROADCAST_API");

        if let Some(addr) = addr {
            spawn(start_ws_server(addr, rx, version));
        }

        Self { tx }
    }
}

async fn start_ws_server(addr: &str, rx: mpsc::UnboundedReceiver<LoaLogsMessageType>, version: String) {
    let rx = Arc::new(Mutex::new(rx)); 
    let app = Router::new().route("/ws", any(move |ws| ws_handler(ws, rx, version)));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();

    let result = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    ).await;

    if let Err(err) = result {
        warn!("Could not start broadcast server: {err}");
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    rx: Arc<Mutex<mpsc::UnboundedReceiver<LoaLogsMessageType>>>,
    version: String
) -> impl IntoResponse {  
    ws.on_upgrade(move |socket| handle_socket(socket, rx, version))
}

async fn handle_socket(mut socket: WebSocket, rx: Arc<Mutex<mpsc::UnboundedReceiver<LoaLogsMessageType>>>, version: String) {
    let mut rx = rx.lock().await;
    
    while let Some(data) = rx.recv().await {
        let message = LoaLogsMessage {
            version: &version,
            data
        };
        let message = serde_json::to_string(&message).unwrap();
        let message = Message::from(message);
        if socket.send(message).await.is_err() {
            warn!("Could not broadcast packet");
            break;
        }
    }
}