use anyhow::Result;
use nineveh_formats::ipc::{IPCClientToServerMessage, IPCServerToClientMessage};
use rfd::{MessageButtons, MessageDialog, MessageLevel};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tauri::{AppHandle, Emitter, Listener};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

mod ipc;

const NINEVEH_ENDPOINT: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 6971);

fn error_and_exit(title: &str, description: &str) -> ! {
    MessageDialog::new()
        .set_title(title)
        .set_description(description)
        .set_level(MessageLevel::Error)
        .set_buttons(MessageButtons::Ok)
        .show();
    std::process::exit(1);
}

/// Handle messages to/from Nineveh IPC server, update frontend about connection status, etc.
/// The app_* channels are for forwarding messages to/from the Tauri app,
/// while the conn_* channels are for communicating with the Nineveh IPC server. This loop will
/// forward only packet-related messages to the app, while other control messages are handled here.
async fn handle_nineveh_ipc_messages(
    app: AppHandle,
    mut app_rx: UnboundedReceiver<IPCClientToServerMessage>,
    app_tx: UnboundedSender<IPCServerToClientMessage>,
    conn_tx: UnboundedSender<IPCClientToServerMessage>,
    mut conn_rx: UnboundedReceiver<IPCServerToClientMessage>,
    mut conn_handle: JoinHandle<()>,
) {
    let active_connections = Arc::new(Mutex::new(Vec::new()));

    let app_responder = app.clone();
    let conns = active_connections.clone();
    app.listen("nineveh-update-request", move |_event| {
        // send current connection state to app when it boots
        app_responder
            .emit("nineveh-update", &*conns.lock().unwrap())
            .unwrap();
    });

    loop {
        tokio::select! {
            // Handle connection closure
            _ = &mut conn_handle => {
                log::warn!("Nineveh IPC connection closed");
                error_and_exit(
                    "Backend Disconnected",
                    "The process that LOA Logs uses to monitor game traffic has disconnected or crashed. Please restart LOA Logs.",
                );
            }

            // Forward messages from app to Nineveh IPC connection
            Some(message) = app_rx.recv() => {
                let _ = conn_tx.send(message);
            }

            // Forward messages from Nineveh IPC connection to app
            Some(message) = conn_rx.recv() => {
                match &message {
                    IPCServerToClientMessage::PacketReceived { .. } => {
                        let _ = app_tx.send(message);
                        continue;
                    },
                    IPCServerToClientMessage::HandshakeAck => {
                        continue;
                    },
                    IPCServerToClientMessage::Connected { connections } => {
                        let mut active_connections = active_connections.lock().unwrap();
                        active_connections.clear();
                        active_connections.extend(connections.iter().cloned());
                    },
                    IPCServerToClientMessage::NewConnection { info } => {
                        let mut active_connections = active_connections.lock().unwrap();
                        active_connections.push(info.clone());
                    },
                    IPCServerToClientMessage::ConnectionClosed { id } => {
                        let mut active_connections = active_connections.lock().unwrap();
                        active_connections.retain(|c| &c.id != id);
                    },
                };

                // send new connection state to app
                app.emit("nineveh-update", &*active_connections.lock().unwrap()).unwrap();
            }
        }
    }
}

pub type NinevehIPCPair = (
    UnboundedSender<IPCClientToServerMessage>,
    UnboundedReceiver<IPCServerToClientMessage>,
);

/// Attempt to connect to an existing Nineveh IPC server or start a new one if not found. Returns
/// channels for receiving packet events from Nineveh and sending commands to it. Connections will
/// be automatically managed and synchronized with the frontend.
pub async fn setup_nineveh(app: AppHandle) -> Result<NinevehIPCPair> {
    let (from_tx, from_rx) = tokio::sync::mpsc::unbounded_channel();
    let (to_tx, to_rx) = tokio::sync::mpsc::unbounded_channel();

    // try to connect to existing server
    if let Ok((rx, tx, handle)) = ipc::connect_to_nineveh(NINEVEH_ENDPOINT).await {
        log::info!("Connected to existing Nineveh IPC server");
        tokio::spawn(handle_nineveh_ipc_messages(
            app, from_rx, to_tx, rx, tx, handle,
        ));
        return Ok((from_tx, to_rx));
    }

    // no existing server; check if there's a nineveh.exe running, in which case
    // we'll want to exit as we can't kill the process ourselves (it's running as
    // administrator), but clearly it's misbehaving if we can't connect to it.
    let system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing().without_tasks()),
    );
    let nineveh_running = system
        .processes()
        .values()
        .any(|p| p.name().eq_ignore_ascii_case("nineveh.exe"));
    if nineveh_running {
        log::error!(
            "Nineveh process is already running but IPC connection could not be established."
        );
        error_and_exit(
            "Backend Unresponsive",
            "The process that LOA Logs uses to monitor game traffic is already running, but it is unresponsive. Please manually stop the 'nineveh.exe' process, then restart LOA Logs.",
        );
    }

    // start new nineveh process
    log::info!("Starting new Nineveh IPC server process");
    // let nineveh_path = std::env::current_exe()
    //     .expect("could not get current exe")
    //     .parent()
    //     .expect("could not get exe parent")
    //     .join("nineveh.exe");
    let nineveh_path =
        "C:\\Users\\thijs\\Documents\\Projects\\LostArk\\nineveh\\target\\release\\nineveh.exe";
    let mut command = tokio::process::Command::new(nineveh_path);
    command.arg("--ipc-port").arg("6971");

    // forward standard output and error to our own process for logging
    command.stdout(std::process::Stdio::inherit());
    command.stderr(std::process::Stdio::inherit());

    // launch command, then keep attempting to connect until either we have a success, or the
    // child process exits.
    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to start Nineveh process: {}", e);
            error_and_exit(
                "Backend Failed to Start",
                "The process that LOA Logs uses to monitor game traffic failed to start. Please ensure that your antivirus or security software is not blocking LOA Logs from running.",
            );
        }
    };
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        match ipc::connect_to_nineveh(NINEVEH_ENDPOINT).await {
            Ok((rx, tx, handle)) => {
                log::info!("Connected to newly started Nineveh IPC server");
                tokio::spawn(handle_nineveh_ipc_messages(
                    app, from_rx, to_tx, rx, tx, handle,
                ));
                return Ok((from_tx, to_rx));
            }
            Err(e) => {
                log::info!("Failed to connect to Nineveh IPC server: {}", e);
            }
        };

        match child.try_wait()? {
            Some(status) => {
                log::error!(
                    "Nineveh process exited unexpectedly with status: {}",
                    status
                );
                error_and_exit(
                    "Backend Failed to Start",
                    r#"The process that LOA Logs uses to monitor game traffic failed to start. Please ensure that your antivirus or security software is not blocking LOA Logs from running."#,
                );
            }
            None => {
                // still running, continue trying
            }
        }
    }
}
