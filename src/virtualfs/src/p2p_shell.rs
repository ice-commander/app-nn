use client_core::NetCmd;
use ic_platform::terminal::PtySession;
use nodeinnet_p2p::P2pMessage;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use tokio::sync::mpsc;

fn sessions() -> &'static Mutex<HashMap<String, mpsc::Sender<Vec<u8>>>> {
    static SESSIONS: OnceLock<Mutex<HashMap<String, mpsc::Sender<Vec<u8>>>>> = OnceLock::new();
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn feed_output(resource_id: &str, data: Vec<u8>) {
    let guard = sessions().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(tx) = guard.get(resource_id) {
        let _ = tx.try_send(data);
    }
}

pub fn open_p2p_shell(
    net_tx: mpsc::Sender<NetCmd>,
    peer_id: String,
    resource_id: String,
    rows: u16,
    cols: u16,
) -> PtySession {
    let (input_tx, mut input_rx) = mpsc::channel::<Vec<u8>>(256);
    let (output_tx, output_rx) = mpsc::channel::<Vec<u8>>(256);
    let (resize_tx, mut resize_rx) = mpsc::channel::<(u16, u16)>(16);

    sessions()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(resource_id.clone(), output_tx);

    let start = [
        P2pMessage::StartTerminal { resource_id: resource_id.clone() },
        P2pMessage::TerminalResize { resource_id: resource_id.clone(), rows, cols },
    ];
    for msg in start {
        let _ = net_tx.try_send(NetCmd::SendToPeer(peer_id.clone(), msg));
    }

    {
        let net_tx = net_tx.clone();
        let peer_id = peer_id.clone();
        let resource_id = resource_id.clone();
        std::thread::spawn(move || {
            while let Some(data) = input_rx.blocking_recv() {
                let msg = P2pMessage::TerminalInput { resource_id: resource_id.clone(), data };
                if net_tx.blocking_send(NetCmd::SendToPeer(peer_id.clone(), msg)).is_err() {
                    return;
                }
            }
            sessions()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(&resource_id);
            let stop = P2pMessage::StopTerminal { resource_id };
            let _ = net_tx.blocking_send(NetCmd::SendToPeer(peer_id, stop));
        });
    }

    std::thread::spawn(move || {
        while let Some((rows, cols)) = resize_rx.blocking_recv() {
            let msg = P2pMessage::TerminalResize { resource_id: resource_id.clone(), rows, cols };
            if net_tx.blocking_send(NetCmd::SendToPeer(peer_id.clone(), msg)).is_err() {
                return;
            }
        }
    });

    PtySession { input_tx, output_rx, resize_tx }
}
