use std::net::SocketAddr;

use anyhow::{Result, anyhow};
use log::error;
use nineveh_formats::ipc::{
    IPCClientToServerMessage, IPCServerToClientMessage, PacketSubscription,
};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

/// Attempt to connect to the IPC server at the given address, and do an initial
/// handshake. If successful, returns a tuple containing the packet read and write halves,
/// as well as a join handle for the read loop task. The returned join handle will finish
/// when the connection is closed or an error occurs.
pub async fn connect_to_nineveh(
    endpoint: SocketAddr,
) -> Result<(
    UnboundedSender<IPCClientToServerMessage>,
    UnboundedReceiver<IPCServerToClientMessage>,
    JoinHandle<()>,
)> {
    let socket = tokio::net::TcpSocket::new_v4()?;
    let stream = socket.connect(endpoint).await?;
    let (mut read_half, mut write_half) = stream.into_split();

    // Send handshake message
    let handshake_msg = IPCClientToServerMessage::Handshake {
        subscription: PacketSubscription::MODIFY_ALL,
    };
    nineveh_formats::io::write(&mut write_half, &handshake_msg).await?;

    // Await handshake response, expect it within a timeout
    match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        nineveh_formats::io::read(&mut read_half),
    )
    .await
    {
        Ok(Ok(IPCServerToClientMessage::HandshakeAck {})) => {
            // Handshake successful
        }
        Ok(Ok(_)) => {
            return Err(anyhow!("Unexpected message during handshake"));
        }
        Ok(Err(e)) => {
            return Err(anyhow!("Failed to read handshake response: {}", e));
        }
        Err(_) => {
            return Err(anyhow!("Handshake response timed out"));
        }
    };

    let (incoming_tx, incoming_rx) = tokio::sync::mpsc::unbounded_channel();
    let (outgoing_tx, mut outgoing_rx) = tokio::sync::mpsc::unbounded_channel();

    let handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Read incoming messages
                result = nineveh_formats::io::read(&mut read_half) => {
                    match result {
                        Ok(msg) => {
                            if incoming_tx.send(msg).is_err() {
                                error!("Failed to send incoming message to channel");
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Error reading IPC message: {}", e);
                            break;
                        }
                    }
                }

                // Write outgoing messages
                Some(msg) = outgoing_rx.recv() => {
                    if let Err(e) = nineveh_formats::io::write(&mut write_half, &msg).await {
                        error!("Error writing IPC message: {}", e);
                        break;
                    }
                }
            }
        }
    });

    Ok((outgoing_tx, incoming_rx, handle))
}
