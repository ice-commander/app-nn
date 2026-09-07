pub mod boot;
pub mod client;

use nodeinnet_p2p::{NodeInfo, P2pMessage};
use std::collections::HashMap;

pub fn shared_terminals(nodes: &[NodeInfo]) -> HashMap<String, String> {
    nodes
        .iter()
        .filter_map(|node| {
            node.resources
                .iter()
                .find(|r| {
                    r.resource_type == nodeinnet_p2p::ResourceType::Terminal && r.is_active
                })
                .map(|r| (node.id.clone(), r.id.clone()))
        })
        .collect()
}

pub fn on_nodes(nodes: &[NodeInfo]) {
    virtualfs::p2p_rpc::set_peer_terminals(shared_terminals(nodes));
}

pub fn pending_id(msg: &P2pMessage) -> Option<uuid::Uuid> {
    match msg {
        P2pMessage::EntriesResponse { request_id, .. }
        | P2pMessage::MetadataResponse { request_id, .. }
        | P2pMessage::CreateDirectoryResponse { request_id, .. }
        | P2pMessage::DeleteEntryResponse { request_id, .. }
        | P2pMessage::RenameEntryResponse { request_id, .. }
        | P2pMessage::SetPermissionsResponse { request_id, .. } => Some(*request_id),
        P2pMessage::FileTransferComplete { transfer_id, .. }
        | P2pMessage::FileTransferResponse { transfer_id, .. } => Some(*transfer_id),
        _ => None,
    }
}

pub fn on_message(msg: &P2pMessage) -> bool {
    if let P2pMessage::TerminalOutput { resource_id, data } = msg {
        virtualfs::p2p_shell::feed_output(resource_id, data.clone());
        return true;
    }

    let Some(id) = pending_id(msg) else {
        return false;
    };
    let waiting = {
        let pending = web_davserver::get_pending_requests();
        let guard = pending.lock().unwrap_or_else(|e| e.into_inner());
        guard.get(&id).cloned()
    };
    match waiting {
        Some(tx) => {
            let _ = tx.send(msg.clone());
            true
        }
        None => false,
    }
}

pub fn peer_shell(
    peer_id: &str,
    net_tx: tokio::sync::mpsc::Sender<client_core::NetCmd>,
    rows: u16,
    cols: u16,
) -> Option<ic_platform::terminal::PtySession> {
    let terminal_id = virtualfs::p2p_rpc::peer_terminal(peer_id)?;
    Some(virtualfs::p2p_shell::open_p2p_shell(
        net_tx,
        peer_id.to_string(),
        terminal_id,
        rows,
        cols,
    ))
}

pub fn on_progress(transfer_id: uuid::Uuid, bytes_read: u64) {
    virtualfs::p2p_rpc::fire_download_progress(transfer_id, bytes_read);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resource(id: &str, kind: nodeinnet_p2p::ResourceType, active: bool) -> nodeinnet_p2p::SharedResource {
        nodeinnet_p2p::SharedResource {
            id: id.to_string(),
            name: "res".to_string(),
            resource_type: kind,
            config: None,
            is_active: active,
            session_token: None,
        }
    }

    fn node(id: &str, resources: Vec<nodeinnet_p2p::SharedResource>) -> NodeInfo {
        NodeInfo {
            id: id.to_string(),
            name: id.to_string(),
            os: "linux".to_string(),
            version: "0.0.0".to_string(),
            app_type: "gui".to_string(),
            build_type: "deb".to_string(),
            public_key: String::new(),
            resources,
            is_online: true,
            last_used: 0,
            is_temporary: false,
        }
    }

    fn entries_response(request_id: uuid::Uuid) -> P2pMessage {
        P2pMessage::EntriesResponse {
            request_id,
            resource_id: "fs-1".to_string(),
            path: "/".to_string(),
            directories: Vec::new(),
            files: Vec::new(),
            directories_with_dates: None,
            files_with_dates: None,
            directories_permissions: None,
            files_permissions: None,
        }
    }

    #[test]
    fn a_peer_sharing_a_terminal_is_remembered_by_its_resource_id() {
        let nodes = vec![node("peer", vec![resource("term-1", nodeinnet_p2p::ResourceType::Terminal, true)])];
        assert_eq!(shared_terminals(&nodes).get("peer").map(String::as_str), Some("term-1"));
    }

    #[test]
    fn a_terminal_switched_off_does_not_count() {
        let nodes = vec![node("peer", vec![resource("term-1", nodeinnet_p2p::ResourceType::Terminal, false)])];
        assert!(shared_terminals(&nodes).is_empty());
    }

    #[test]
    fn a_peer_sharing_only_files_offers_no_terminal() {
        let nodes = vec![node("peer", vec![resource("fs-1", nodeinnet_p2p::ResourceType::Filesystem, true)])];
        assert!(shared_terminals(&nodes).is_empty());
    }

    #[test]
    fn every_answer_carrying_a_request_id_is_recognised() {
        let id = uuid::Uuid::new_v4();
        let answers = [
            entries_response(id),
            P2pMessage::CreateDirectoryResponse {
                request_id: id,
                resource_id: "fs-1".to_string(),
                parent_path: "/".to_string(),
                result: Ok(()),
            },
            P2pMessage::DeleteEntryResponse {
                request_id: id,
                resource_id: "fs-1".to_string(),
                parent_path: "/".to_string(),
                result: Ok(()),
            },
            P2pMessage::RenameEntryResponse {
                request_id: id,
                resource_id: "fs-1".to_string(),
                parent_path: "/".to_string(),
                result: Ok(()),
            },
        ];
        for msg in answers {
            assert_eq!(pending_id(&msg), Some(id), "{msg:?}");
        }
    }

    #[test]
    fn a_message_nobody_waits_for_is_left_alone() {
        assert!(!on_message(&entries_response(uuid::Uuid::new_v4())));
    }

    #[test]
    fn terminal_output_is_always_consumed_here() {
        let msg = P2pMessage::TerminalOutput {
            resource_id: "term-1".to_string(),
            data: b"hello".to_vec(),
        };
        assert!(on_message(&msg));
    }
}
