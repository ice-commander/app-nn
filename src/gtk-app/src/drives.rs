#[cfg(feature = "nodeinnet")]
use client_core;
#[cfg(not(feature = "nodeinnet"))]
use crate::core::client_core;

use crate::connection_manager::FtpConnection;
use virtualfs::utils::{get_drives, DriveInfo};
use panel_router::PanelRouter;
use gtk::prelude::*;
use std::rc::Rc;

#[derive(Clone)]
pub enum AppDriveItem {
    RootFs,
    UserHome,
    LocalDrive(String),
    Volume(gtk::gio::Volume),
    NetConnection(FtpConnection),
    RemotePeer {
        peer_id: String,
        resource_id: String,
        name: String,
    },
}

#[derive(Clone)]
pub struct AppDrive {
    pub item: AppDriveItem,
    pub name: String,
    pub subtitle: String,
    pub icon: String,      // e.g., "/com/icecommander/gtk/ssd.svg"
    pub key: String,       // e.g., "local_fs:/", "ftp://..."
    pub is_favorite: bool,
    pub is_online: bool,
    #[allow(dead_code)]
    pub drive_info: Option<DriveInfo>,
}

pub enum DriveActivation {
    Shown,
    NeedsAsyncMount(gtk::gio::Volume),
    Noop,
}

#[derive(serde::Deserialize, Clone, Debug)]
struct SavedPeerConnection {
    name: String,
    peer_id: String,
}

impl AppDrive {
    #[allow(dead_code)] // distinct from the `DriveInfo::is_mounted` field the other crates read
    pub fn is_mounted(&self) -> bool {
        match &self.item {
            AppDriveItem::RootFs | AppDriveItem::UserHome | AppDriveItem::LocalDrive(_) => true,
            AppDriveItem::Volume(vol) => vol.get_mount().is_some(),
            AppDriveItem::NetConnection(_) => false,
            AppDriveItem::RemotePeer { .. } => false,
        }
    }
}

pub fn get_all_app_drives(
    config: &client_config::AppConfig,
    online_nodes: &[nodeinnet_p2p::NodeInfo],
    active_p2p: Option<(String, String)>,
) -> Vec<AppDrive> {
    let mut drives = Vec::new();
    let favorites = config.get::<Vec<String>>("ui.favorites").unwrap_or_default();

    let root_key = "local_fs:/".to_string();
    drives.push(AppDrive {
        item: AppDriveItem::RootFs,
        name: crate::i18n::tr("drives.system_root").to_string(),
        subtitle: "/".to_string(),
        icon: "/com/icecommander/gtk/home.svg".to_string(),
        key: root_key.clone(),
        is_favorite: favorites.contains(&root_key),
        is_online: true,
        drive_info: None,
    });

    let home_key = "local_fs:~".to_string();
    let home_path = dirs::home_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    drives.push(AppDrive {
        item: AppDriveItem::UserHome,
        name: crate::i18n::tr("drives.user_home").to_string(),
        subtitle: home_path,
        icon: "/com/icecommander/gtk/at-home.svg".to_string(),
        key: home_key.clone(),
        is_favorite: favorites.contains(&home_key),
        is_online: true,
        drive_info: None,
    });

    for drive in get_drives() {
        if drive.is_mounted {
            let key = format!("local_fs:{}", drive.path);
            drives.push(AppDrive {
                item: AppDriveItem::LocalDrive(drive.path.clone()),
                name: drive.name.clone(),
                subtitle: drive.path.clone(),
                icon: "/com/icecommander/gtk/ssd.svg".to_string(),
                is_favorite: favorites.contains(&key),
                is_online: true,
                key,
                drive_info: Some(drive),
            });
        }
    }

    let monitor = gtk::gio::VolumeMonitor::get();
    for volume in monitor.volumes() {
        if volume.can_mount() && volume.get_mount().is_none() {
            let name = volume.name().to_string();
            let key = name.clone();
            drives.push(AppDrive {
                item: AppDriveItem::Volume(volume.clone()),
                name: name.clone(),
                subtitle: crate::i18n::tr("drives.not_mounted").to_string(),
                icon: "/com/icecommander/gtk/ssd.svg".to_string(),
                is_favorite: favorites.contains(&key),
                is_online: false,
                key,
                drive_info: Some(DriveInfo {
                    path: name.clone(),
                    name,
                    is_mounted: false,
                    can_eject: volume.can_eject(),
                    volume: Some(volume),
                    mount: None,
                }),
            });
        }
    }

    let all_conns: Vec<FtpConnection> = config.get("ui.ftp_connections").unwrap_or_default();
    for conn in &all_conns {
        let is_webdav = conn.protocol.to_uppercase() == "WEBDAV";
        let key = if is_webdav {
            format!("webdav://{}@{}", conn.user, conn.host)
        } else {
            format!(
                "{}://{}@{}:{}",
                conn.protocol.to_lowercase(),
                conn.user,
                conn.host,
                conn.port
            )
        };
        let icon = if is_webdav {
            "/com/icecommander/gtk/netdrive.svg".to_string()
        } else {
            "/com/icecommander/gtk/ftp.svg".to_string()
        };
        let subtitle = format!("{}://{}", conn.protocol.to_lowercase(), conn.host);

        drives.push(AppDrive {
            item: AppDriveItem::NetConnection(conn.clone()),
            name: conn.name.clone(),
            subtitle,
            icon,
            is_favorite: favorites.contains(&key),
            is_online: true,
            key,
            drive_info: None,
        });
    }

    let peer_conns: Vec<SavedPeerConnection> = config.get("ui.peers").unwrap_or_default();
    let mut added_p2p_keys = std::collections::HashSet::new();

    let active_key = active_p2p
        .as_ref()
        .map(|(peer_id, resource_id)| format!("p2p://{}@{}", peer_id, resource_id));

    if let Some((peer_id, resource_id)) = active_p2p {
        let key = format!("p2p://{}@{}", peer_id, resource_id);
        let name = get_p2p_drive_name(&peer_id, &resource_id, config);
        added_p2p_keys.insert(key.clone());
        drives.push(AppDrive {
            item: AppDriveItem::RemotePeer {
                peer_id: peer_id.clone(),
                resource_id: resource_id.clone(),
                name: name.clone(),
            },
            name,
            subtitle: format!("Resource ID: {} | Peer ID: {}", resource_id, peer_id),
            icon: "/com/icecommander/gtk/connect.svg".to_string(),
            is_favorite: favorites.contains(&key),
            is_online: true,
            key,
            drive_info: None,
        });
    }

    for peer in &peer_conns {
        if let Some(node) = online_nodes.iter().find(|n| n.id == peer.peer_id && n.is_online) {
            let fs_resources: Vec<&nodeinnet_p2p::SharedResource> = node
                .resources
                .iter()
                .filter(|r| r.resource_type == nodeinnet_p2p::ResourceType::Filesystem)
                .collect();

            if fs_resources.is_empty() {
                let key = format!("p2p://{}", peer.peer_id);
                if !added_p2p_keys.contains(&key) {
                    added_p2p_keys.insert(key.clone());
                    drives.push(AppDrive {
                        item: AppDriveItem::RemotePeer {
                            peer_id: peer.peer_id.clone(),
                            resource_id: String::new(),
                            name: peer.name.clone(),
                        },
                        name: peer.name.clone(),
                        subtitle: peer.peer_id.clone(),
                        icon: "/com/icecommander/gtk/connect.svg".to_string(),
                        is_favorite: favorites.contains(&key),
                        is_online: true,
                        key,
                        drive_info: None,
                    });
                }
            } else {
                for fs_res in fs_resources {
                    let key = format!("p2p://{}@{}", peer.peer_id, fs_res.id);
                    if !added_p2p_keys.contains(&key) {
                        added_p2p_keys.insert(key.clone());
                        let name = format!("{} ({})", fs_res.name, peer.name);
                        drives.push(AppDrive {
                            item: AppDriveItem::RemotePeer {
                                peer_id: peer.peer_id.clone(),
                                resource_id: fs_res.id.clone(),
                                name: name.clone(),
                            },
                            name,
                            subtitle: format!("Resource ID: {} | Peer ID: {}", fs_res.id, peer.peer_id),
                            icon: "/com/icecommander/gtk/connect.svg".to_string(),
                            is_favorite: favorites.contains(&key),
                            is_online: true,
                            key,
                            drive_info: None,
                        });
                    }
                }
            }
        } else {
            let key = format!("p2p://{}", peer.peer_id);
            if !added_p2p_keys.contains(&key) {
                added_p2p_keys.insert(key.clone());
                drives.push(AppDrive {
                    item: AppDriveItem::RemotePeer {
                        peer_id: peer.peer_id.clone(),
                        resource_id: String::new(),
                        name: peer.name.clone(),
                    },
                    name: crate::i18n::trf("drives.offline_suffix", &[("name", &*(peer.name.clone()).to_string())]).to_string(),
                    subtitle: peer.peer_id.clone(),
                    icon: "/com/icecommander/gtk/connect.svg".to_string(),
                    is_favorite: favorites.contains(&key),
                    is_online: false,
                    key,
                    drive_info: None,
                });
            }
        }
    }

    for node in online_nodes {
        if !node.is_online {
            continue;
        }
        let is_saved = peer_conns.iter().any(|p| p.peer_id == node.id);
        if is_saved {
            continue;
        }

        for fs_res in &node.resources {
            if fs_res.resource_type == nodeinnet_p2p::ResourceType::Filesystem {
                let key = format!("p2p://{}@{}", node.id, fs_res.id);
                if Some(&key) == active_key.as_ref() {
                    continue;
                }
                if !added_p2p_keys.contains(&key) {
                    added_p2p_keys.insert(key.clone());
                    let node_display_name = if node.name.is_empty() {
                        crate::i18n::tr("drives.unnamed_device").to_string()
                    } else {
                        node.name.clone()
                    };
                    let name = format!("{} ({})", fs_res.name, node_display_name);
                    drives.push(AppDrive {
                        item: AppDriveItem::RemotePeer {
                            peer_id: node.id.clone(),
                            resource_id: fs_res.id.clone(),
                            name: name.clone(),
                        },
                        name,
                        subtitle: format!("Resource ID: {} | Peer ID: {}", fs_res.id, node.id),
                        icon: "/com/icecommander/gtk/connect.svg".to_string(),
                        is_favorite: favorites.contains(&key),
                        is_online: true,
                        key,
                        drive_info: None,
                    });
                }
            }
        }
    }

    drives
}

pub fn activate_drive_item(
    item: &AppDriveItem,
    router: &Rc<PanelRouter>,
    #[allow(unused_variables)] net_tx: &crate::core::NetCmdSender,
) -> DriveActivation {
    match item {
        AppDriveItem::RootFs => {
            router.open_local_path("/".to_string());
            DriveActivation::Shown
        }
        AppDriveItem::UserHome => {
            let home_path = dirs::home_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            router.open_local_path(home_path);
            DriveActivation::Shown
        }
        AppDriveItem::LocalDrive(path) => {
            router.open_local_path(path.clone());
            DriveActivation::Shown
        }
        AppDriveItem::Volume(vol) => DriveActivation::NeedsAsyncMount(vol.clone()),
        AppDriveItem::NetConnection(conn) => {
            let conn = &crate::secret_store::opened(conn);
            let rpath = conn.remote_path.clone().unwrap_or_else(|| "/".to_string());

            match conn.protocol.to_uppercase().as_str() {
                "FTP" => {
                    let ftp_rpc = Rc::new(virtualfs::ftp_rpc::LocalFtpRpc {
                        name: conn.name.clone(),
                        host: conn.host.clone(),
                        port: conn.port,
                        user: conn.user.clone(),
                        pass: conn.pass.clone().unwrap_or_default(),
                        ftp_session: std::sync::Arc::new(std::sync::Mutex::new(None)),
                    });
                    router.mount_provider(ftp_rpc, "ftp", rpath);
                }
                "WEBDAV" => {
                    let webdav_rpc = Rc::new(virtualfs::webdav_rpc::LocalWebDavRpc {
                        name: conn.name.clone(),
                        url: conn.host.clone(),
                        user: if conn.user.is_empty() {
                            None
                        } else {
                            Some(conn.user.clone())
                        },
                        pass: conn.pass.clone(),
                        remote_path: conn.remote_path.clone(),
                    });
                    router.mount_provider(webdav_rpc, "webdav", rpath);
                }
                _ => {
                    let sftp_rpc = Rc::new(virtualfs::sftp_rpc::LocalSftpRpc {
                        name: conn.name.clone(),
                        host: conn.host.clone(),
                        port: conn.port,
                        user: conn.user.clone(),
                        pass: conn.pass.clone(),
                        auth_type: conn
                            .auth_type
                            .clone()
                            .unwrap_or_else(|| "password".to_string()),
                        key_path: conn.key_path.clone(),
                        passphrase: conn.passphrase.clone(),
                        use_tunnel: conn.use_tunnel,
                        tunnel_host: conn.tunnel_host.clone(),
                        tunnel_port: conn.tunnel_port,
                        tunnel_user: conn.tunnel_user.clone(),
                        tunnel_auth_type: conn.tunnel_auth_type.clone(),
                        tunnel_pass: conn.tunnel_pass.clone(),
                        tunnel_key_path: conn.tunnel_key_path.clone(),
                        tunnel_passphrase: conn.tunnel_passphrase.clone(),
                        sftp_session: std::sync::Arc::new(std::sync::Mutex::new(None)),
                        tunnel: std::sync::Arc::new(std::sync::Mutex::new(None)),
                    });
                    router.mount_provider(sftp_rpc, "sftp", rpath);
                }
            }
            DriveActivation::Shown
        }
        #[cfg(feature = "nodeinnet")]
        AppDriveItem::RemotePeer {
            peer_id,
            resource_id,
            ..
        } => {
            let remote_rpc = Rc::new(virtualfs::p2p_rpc::RemoteFileSystemRpc {
                net_tx: net_tx.clone(),
                resource_id: resource_id.clone(),
                peer_id: peer_id.clone(),
            });
            let _ = net_tx.try_send(client_core::NetCmd::Call(peer_id.clone()));
            router.mount_provider(remote_rpc, resource_id.clone(), "/".to_string());
            DriveActivation::Shown
        }
        #[cfg(not(feature = "nodeinnet"))]
        AppDriveItem::RemotePeer { .. } => DriveActivation::Noop,
    }
}

fn get_p2p_drive_name(peer_id: &str, resource_id: &str, config: &client_config::AppConfig) -> String {
    let peer_conns = config.get::<Vec<SavedPeerConnection>>("ui.peers").unwrap_or_default();
    let resolved_peer_name = if let Some(peer) = peer_conns.iter().find(|p| p.peer_id == peer_id) {
        Some(peer.name.clone())
    } else if let Some(online_name) = nodeinnet_p2p::get_known_peer_name(peer_id) {
        Some(online_name)
    } else {
        None
    };

    let resolved_resource_name = nodeinnet_p2p::get_known_resource_name(peer_id, resource_id)
        .unwrap_or_else(|| resource_id.to_string());

    if let Some(peer_name) = resolved_peer_name {
        format!("{} ({})", resolved_resource_name, peer_name)
    } else {
        let short_id = if peer_id.len() > 12 {
            format!("{}...{}", &peer_id[0..6], &peer_id[peer_id.len() - 6..])
        } else {
            peer_id.to_string()
        };
        format!("{} ({})", resolved_resource_name, short_id)
    }
}
