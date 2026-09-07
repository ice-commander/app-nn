use crate::boot;
use nodeinnet_p2p::{NodeInfo, P2pMessage};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::mpsc;

pub enum NetEvent {
    Nodes(Vec<NodeInfo>),
    Message(P2pMessage),
    Progress { transfer_id: uuid::Uuid, bytes_read: u64 },
}

struct Forwarder {
    tx: mpsc::UnboundedSender<NetEvent>,
}

#[async_trait::async_trait]
impl client_core::AppEventHandler for Forwarder {
    async fn on_log(&self, _msg: String) {}
    async fn on_connected(&self) {}
    async fn on_disconnected(&self) {}
    async fn on_update_nodes(&self, nodes: Vec<NodeInfo>) {
        let _ = self.tx.send(NetEvent::Nodes(nodes));
    }
    async fn on_download_complete(&self, _path: PathBuf) {}
    async fn on_p2p_message(&self, msg: P2pMessage) {
        let _ = self.tx.send(NetEvent::Message(msg));
    }
    async fn on_p2p_connected(&self, _peer_id: String) {}
    async fn on_p2p_disconnected(&self, _peer_id: String) {}
    async fn on_local_p2p_event(&self, event: p2p_node::LocalP2pEvent) {
        if let p2p_node::LocalP2pEvent::TransferProgress { transfer_id, bytes_read, .. } = event {
            let _ = self.tx.send(NetEvent::Progress { transfer_id, bytes_read });
        }
    }
}

#[derive(Clone)]
pub struct P2p {
    pub net_tx: mpsc::Sender<client_core::NetCmd>,
    pub online_nodes: Rc<RefCell<Vec<NodeInfo>>>,
    pub my_id: String,
    pub node_info: NodeInfo,
}

pub fn start(
    config: &client_config::AppConfig,
    device: &ic_model::DeviceInfo,
) -> Option<(P2p, mpsc::UnboundedReceiver<NetEvent>)> {
    if !boot::enabled(config) {
        return None;
    }
    let identity = boot::identity(config);
    boot::apply_api_endpoint(config);

    let node_info = NodeInfo {
        id: device.id.clone(),
        name: device.name.clone(),
        os: device.os.clone(),
        version: device.version.clone(),
        app_type: device.app_type.clone(),
        build_type: device.build_type.clone(),
        public_key: identity.public_key,
        resources: Vec::new(),
        is_online: true,
        last_used: 0,
        is_temporary: config.get::<bool>("app.is_guest").unwrap_or(false),
    };

    let announced = node_info.clone();
    let (tx, rx) = mpsc::unbounded_channel();
    let net_tx = boot::start(
        config,
        node_info,
        identity.private_key,
        Arc::new(Forwarder { tx }),
    );

    let p2p = P2p {
        net_tx,
        online_nodes: Rc::new(RefCell::new(Vec::new())),
        my_id: device.id.clone(),
        node_info: announced,
    };
    Some((p2p, rx))
}

pub async fn sign_in(
    config: client_config::AppConfig,
    node_info: NodeInfo,
    net_tx: mpsc::Sender<client_core::NetCmd>,
) -> Result<(), String> {
    let token = config
        .get::<String>("app.refresh_token")
        .filter(|t| !t.is_empty())
        .ok_or_else(|| "no stored account: sign in from the desktop app first".to_string())?;

    let response = client_core::auth::refresh_access_token(
        &nodeinnet_p2p::api_base(),
        &token,
        config.turn_region(),
    )
    .await
    .map_err(|e| format!("sign-in failed: {e}"))?;

    config.set("app.premium", response.premium != 0);
    config.save();

    let url = format!(
        "{}?token={}&session_id={}",
        response.ws_url, response.access_token, node_info.id
    );
    net_tx
        .send(client_core::NetCmd::Connect(url, node_info, response.turn))
        .await
        .map_err(|_| "network thread is gone".to_string())
}

pub async fn pump(p2p: P2p, mut rx: mpsc::UnboundedReceiver<NetEvent>) {
    while let Some(event) = rx.recv().await {
        match event {
            NetEvent::Nodes(nodes) => {
                crate::on_nodes(&nodes);
                *p2p.online_nodes.borrow_mut() = nodes;
            }
            NetEvent::Message(msg) => {
                let _ = crate::on_message(&msg);
            }
            NetEvent::Progress { transfer_id, bytes_read } => {
                crate::on_progress(transfer_id, bytes_read);
            }
        }
    }
}
