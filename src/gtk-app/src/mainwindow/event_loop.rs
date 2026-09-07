macro_rules! println {
    ($($arg:tt)*) => {
        if false {
            let _ = format_args!($($arg)*);
        }
    };
}

use gtk::glib;

#[cfg(feature = "nodeinnet")]
use client_core;
#[cfg(not(feature = "nodeinnet"))]
use crate::core::client_core;

#[cfg(feature = "nodeinnet")]
use web_davserver;
#[cfg(not(feature = "nodeinnet"))]
use crate::core::web_davserver;

pub(super) fn start_event_loop(
    ui_rx: std::sync::mpsc::Receiver<crate::core::UiEvent>,
    online_nodes: std::rc::Rc<std::cell::RefCell<Vec<nodeinnet_p2p::NodeInfo>>>,
    selector_updaters: std::rc::Rc<std::cell::RefCell<Vec<std::rc::Rc<dyn Fn()>>>>,
    _left_router: std::rc::Rc<panel_router::PanelRouter>,
    _right_router: std::rc::Rc<panel_router::PanelRouter>,
    login_label: gtk::Label,
    ws_state: std::rc::Rc<std::cell::RefCell<client_core::WsState>>,
    active_dialog_graph: std::rc::Rc<std::cell::RefCell<Option<crate::netgraph::NetGraph>>>,
    my_info: nodeinnet_p2p::NodeInfo,
    config: client_config::AppConfig,
) {
    glib::MainContext::default().spawn_local(async move {
        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            while let Ok(event) = ui_rx.try_recv() {
                match event {
                    crate::core::UiEvent::UpdateNodes(nodes) => {
                        *online_nodes.borrow_mut() = nodes.clone();
                        #[cfg(feature = "nodeinnet")]
                        p2p_runtime::on_nodes(&nodes);

                        println!("============================================================");
                        println!("📡 [WEBSOCKET] Peer list update received (from signaling: {})", nodes.len());

                        let peer_conns: Vec<crate::connection_manager::PeerConnection> = config
                            .get("ui.peers")
                            .unwrap_or_default();

                        println!("--- Saved Peers ---");
                        for peer in &peer_conns {
                            let is_online = nodes.iter().any(|n| n.id == peer.peer_id && n.is_online);
                            println!(
                                "  • Peer: {} | ID: {} | Status: {}",
                                peer.name,
                                peer.peer_id,
                                if is_online { "🟢 ONLINE" } else { "🔴 OFFLINE" }
                            );

                            if is_online {
                                if let Some(node) = nodes.iter().find(|n| n.id == peer.peer_id && n.is_online) {
                                    for res in &node.resources {
                                        let is_fs = res.resource_type == nodeinnet_p2p::ResourceType::Filesystem;
                                        println!(
                                            "    └─ Resource: {} [ID: {}] | Type: {:?} | Will display: {}",
                                            res.name,
                                            res.id,
                                            res.resource_type,
                                            if is_fs { "✅ YES (Filesystem)" } else { "❌ NO (not a Filesystem)" }
                                        );
                                    }
                                }
                            } else {
                                println!("    └─ (peer offline, resources not shown)");
                            }
                        }

                        println!("--- Discovered Peers (unsaved) ---");
                        for node in &nodes {
                            if node.id == my_info.id {
                                println!(
                                    "  • [Self] {} | ID: {} | Status: {}",
                                    node.name,
                                    node.id,
                                    if node.is_online { "🟢 ONLINE" } else { "🔴 OFFLINE" }
                                );
                                continue;
                            }

                            let is_saved = peer_conns.iter().any(|p| p.peer_id == node.id);
                            if !is_saved {
                                println!(
                                    "  • Peer: {} | ID: {} | Status: {}",
                                    node.name,
                                    node.id,
                                    if node.is_online { "🟢 ONLINE" } else { "🔴 OFFLINE" }
                                );
                                if node.resources.is_empty() {
                                    println!("    └─ No resources");
                                }
                                for res in &node.resources {
                                    let is_fs = res.resource_type == nodeinnet_p2p::ResourceType::Filesystem;
                                    let will_display = is_fs && node.is_online;
                                    println!(
                                        "    └─ Resource: {} [ID: {}] | Type: {:?} | Will display: {}",
                                        res.name,
                                        res.id,
                                        res.resource_type,
                                        if will_display { "✅ YES (Filesystem)" } else { "❌ NO" }
                                    );
                                }
                            }
                        }
                        println!("============================================================");

                        for updater in selector_updaters.borrow().iter() {
                            updater();
                        }
                        if let Some(graph) = active_dialog_graph.borrow().as_ref() {
                            graph.update_peers(&nodes, &my_info.id);
                        }
                    }

                    crate::core::UiEvent::WsStateChanged(state) => {
                        *ws_state.borrow_mut() = state;
                        match state {
                            client_core::WsState::Disconnected => {
                                login_label.set_text("Disconnected");
                            }
                            client_core::WsState::Connecting => {
                                login_label.set_text("Connecting...");
                            }
                            client_core::WsState::Connected => {
                                let account_name = config
                                    .get::<String>("app.account_login")
                                    .unwrap_or_else(|| "Connected".to_string());
                                login_label.set_text(&account_name);
                            }
                            client_core::WsState::LocalMesh | client_core::WsState::LocalMeshError => {}
                        }
                    }

                    crate::core::UiEvent::P2pMessageReceived(msg) => {
                        #[cfg(feature = "nodeinnet")]
                        let _ = p2p_runtime::on_message(&msg);
                        let _ = &msg;
                    }

                    #[cfg(feature = "nodeinnet")]
                    crate::core::UiEvent::P2pProgress { transfer_id, bytes_read } => {
                        p2p_runtime::on_progress(transfer_id, bytes_read);
                    }
                    #[cfg(not(feature = "nodeinnet"))]
                    crate::core::UiEvent::P2pProgress { .. } => {}

                    _ => {}
                }
            }
            glib::ControlFlow::Continue
        });
    });
}
