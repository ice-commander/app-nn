#[cfg(feature = "nodeinnet")]
use client_core::{AppEventHandler, WsState};

#[cfg(feature = "nodeinnet")]
pub type NetCmdSender = tokio::sync::mpsc::Sender<client_core::NetCmd>;

#[cfg(not(feature = "nodeinnet"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsState {
    Disconnected,
    Connecting,
    Connected,
    LocalMesh,
    LocalMeshError,
}

#[cfg(not(feature = "nodeinnet"))]
#[derive(Clone, Debug)]
pub struct NetCmdSender;

#[cfg(not(feature = "nodeinnet"))]
impl NetCmdSender {
    pub fn try_send<T>(&self, _msg: T) -> Result<(), &'static str> {
        Ok(())
    }
    pub async fn send<T>(&self, _msg: T) -> Result<(), &'static str> {
        Ok(())
    }
}

#[cfg(not(feature = "nodeinnet"))]
pub mod client_core_mock {
    pub use super::WsState;

    #[derive(Debug, Clone)]
    pub enum NetCmd {
        ReloadResources(Vec<nodeinnet_p2p::SharedResource>),
        UpdateName(String),
        Call(String),
        Disconnect,
        Connect(String, nodeinnet_p2p::NodeInfo, Option<nodeinnet_p2p::TurnCredentials>),
        RegisterUpload {
            peer_id: String,
            transfer_id: uuid::Uuid,
            local_file_path: std::path::PathBuf,
        },
        SendToPeer(String, nodeinnet_p2p::P2pMessage),
    }

    pub mod auth {
        pub async fn refresh_access_token(
            _api_target: &str,
            _refresh_token: &str,
            _region: nodeinnet_p2p::TurnRegion,
        ) -> Result<nodeinnet_p2p::RefreshResponse, String> {
            Err("Authentication disabled".to_string())
        }

        pub async fn login(
            _api_target: &str,
            _login: &str,
            _password: &str,
            _region: nodeinnet_p2p::TurnRegion,
        ) -> Result<nodeinnet_p2p::LoginResponse, String> {
            Err("Authentication disabled".to_string())
        }
    }
}

#[cfg(not(feature = "nodeinnet"))]
pub use client_core_mock as client_core;

#[cfg(not(feature = "nodeinnet"))]
pub mod web_davserver_mock {
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;

    pub type PendingMap =
        HashMap<uuid::Uuid, tokio::sync::mpsc::UnboundedSender<nodeinnet_p2p::P2pMessage>>;

    lazy_static::lazy_static! {
        static ref PENDING: Arc<Mutex<PendingMap>> = Arc::new(Mutex::new(HashMap::new()));
    }

    pub fn get_pending_requests() -> Arc<Mutex<PendingMap>> {
        PENDING.clone()
    }

    pub fn mount_resource(
        _resource_id: String,
        _drive_name: String,
        _net_tx: tokio::sync::mpsc::Sender<nodeinnet_p2p::P2pMessage>,
    ) -> Option<u16> {
        None
    }

    pub fn unmount_all() {}

    pub fn open_in_explorer(_port: u16) {}
}

#[cfg(not(feature = "nodeinnet"))]
pub use web_davserver_mock as web_davserver;


use nodeinnet_p2p::{NodeInfo, P2pMessage};
use std::path::PathBuf;
use std::sync::mpsc;

pub enum UiEvent {
    Connected,
    Disconnected,
    UpdateNodes(Vec<NodeInfo>),
    WsStateChanged(WsState),
    P2pMessageReceived(P2pMessage),
    PeerConnected(String),
    PeerDisconnected(String),
    P2pProgress {
        transfer_id: uuid::Uuid,
        bytes_read: u64,
    },
}

pub struct GtkFmEventHandler {
    pub ui_tx: mpsc::Sender<UiEvent>,
}

#[cfg(feature = "nodeinnet")]
#[async_trait::async_trait]
impl AppEventHandler for GtkFmEventHandler {
    async fn on_log(&self, msg: String) {
        if msg.starts_with("📁 [Config]") || msg.starts_with("📋 [SIGNALING]") {
            println!("{}", msg);
        }
    }
    async fn on_connected(&self) {
        let _ = self.ui_tx.send(UiEvent::Connected);
    }
    async fn on_disconnected(&self) {
        let _ = self.ui_tx.send(UiEvent::Disconnected);
    }
    async fn on_update_nodes(&self, nodes: Vec<NodeInfo>) {
        let _ = self.ui_tx.send(UiEvent::UpdateNodes(nodes));
    }
    async fn on_download_complete(&self, _path: PathBuf) {}

    async fn on_ws_state_changed(&self, state: WsState) {
        let _ = self.ui_tx.send(UiEvent::WsStateChanged(state));
    }

    async fn on_p2p_message(&self, msg: P2pMessage) {
        let _ = self.ui_tx.send(UiEvent::P2pMessageReceived(msg));
    }

    async fn on_p2p_connecting(&self, _peer_id: String) {}
    async fn on_p2p_connected(&self, peer_id: String) {
        let _ = self.ui_tx.send(UiEvent::PeerConnected(peer_id));
    }
    async fn on_p2p_disconnected(&self, peer_id: String) {
        let _ = self.ui_tx.send(UiEvent::PeerDisconnected(peer_id));
    }
    async fn on_p2p_ping_updated(&self, _peer_id: String, _rtt_ms: u64) {}
    async fn on_local_p2p_event(&self, event: p2p_node::LocalP2pEvent) {
        if let p2p_node::LocalP2pEvent::TransferProgress { transfer_id, bytes_read, .. } = event {
            let _ = self.ui_tx.send(UiEvent::P2pProgress { transfer_id, bytes_read });
        }
    }
}
