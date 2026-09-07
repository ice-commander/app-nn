use client_core::{AppEventHandler, NetCmd};
use nodeinnet_p2p::NodeInfo;
use std::sync::Arc;
use tokio::sync::mpsc;

const PRIVATE_KEY: &str = "app.private_key_b64";
const PUBLIC_KEY: &str = "app.public_key_b64";

pub const NAME_KEY: &str = "app-name";
const LEGACY_NAME_KEY: &str = "app.device_name";
const DEVICE_ID: &str = "app.device_id";

pub fn device_id(config: &client_config::AppConfig) -> String {
    if let Some(id) = config.get::<String>(DEVICE_ID).filter(|s| !s.is_empty()) {
        return id;
    }
    let id = uuid::Uuid::new_v4().to_string();
    config.set(DEVICE_ID, &id);
    config.save();
    id
}

pub fn default_device_name() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "IceCommander Client".to_string())
}

pub fn device_name(config: &client_config::AppConfig) -> String {
    config
        .get::<String>(NAME_KEY)
        .or_else(|| config.get::<String>(LEGACY_NAME_KEY))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(default_device_name)
}

pub fn set_device_name(config: &client_config::AppConfig, name: &str) {
    let trimmed = name.trim();
    let value = if trimmed.is_empty() {
        default_device_name()
    } else {
        trimmed.to_string()
    };
    config.set(NAME_KEY, value);
    config.save();
}

pub fn device(
    config: &client_config::AppConfig,
    app_type: &str,
    version: &str,
    build_type: &str,
) -> ic_model::DeviceInfo {
    ic_model::DeviceInfo {
        id: device_id(config),
        name: device_name(config),
        os: std::env::consts::OS.to_string(),
        version: version.to_string(),
        app_type: app_type.to_string(),
        build_type: build_type.to_string(),
    }
}

pub struct Identity {
    pub private_key: String,
    pub public_key: String,
}

pub fn identity(config: &client_config::AppConfig) -> Identity {
    let private = config.get::<String>(PRIVATE_KEY).unwrap_or_default();
    let public = config.get::<String>(PUBLIC_KEY).unwrap_or_default();
    if !private.is_empty() && !public.is_empty() {
        return Identity { private_key: private, public_key: public };
    }

    let (private, public) = nodeinnet_p2p::generate_ed25519_keypair();
    config.set(PRIVATE_KEY, &private);
    config.set(PUBLIC_KEY, &public);
    config.save();
    Identity { private_key: private, public_key: public }
}

pub fn apply_api_endpoint(config: &client_config::AppConfig) {
    if let Some(endpoint) = config.get::<String>("net.api_endpoint") {
        if !endpoint.is_empty() {
            nodeinnet_p2p::set_api_base(&endpoint);
        }
    }
}

pub fn enabled(config: &client_config::AppConfig) -> bool {
    config.get::<bool>("ui.p2p_enabled").unwrap_or(true)
}

pub fn start(
    config: &client_config::AppConfig,
    node_info: NodeInfo,
    private_key: String,
    handler: Arc<dyn AppEventHandler>,
) -> mpsc::Sender<NetCmd> {
    let (net_tx, net_rx) = mpsc::channel::<NetCmd>(100);
    client_core::network::start_network_thread(
        net_rx,
        net_tx.clone(),
        handler,
        node_info,
        private_key,
        Arc::new(client_config::ConfigPeerStore::new(config.clone())),
        config.get::<bool>("net.local_discovery").unwrap_or(true),
    );
    net_tx
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(name: &str) -> client_config::AppConfig {
        let c = client_config::AppConfig::new(&format!("_ice_commander_unit_tests_boot_{name}"));
        c.set(PRIVATE_KEY, &String::new());
        c.set(PUBLIC_KEY, &String::new());
        c
    }

    #[test]
    fn a_device_without_keys_is_given_a_pair_that_survives_a_restart() {
        let c = config("fresh");
        let first = identity(&c);
        assert!(!first.private_key.is_empty() && !first.public_key.is_empty());

        let reopened = client_config::AppConfig::new("_ice_commander_unit_tests_boot_fresh");
        let second = identity(&reopened);
        assert_eq!(first.private_key, second.private_key);
        assert_eq!(first.public_key, second.public_key);
    }

    #[test]
    fn an_existing_pair_is_kept_as_it_is() {
        let c = config("kept");
        c.set(PRIVATE_KEY, &"stored-private".to_string());
        c.set(PUBLIC_KEY, &"stored-public".to_string());

        let id = identity(&c);
        assert_eq!(id.private_key, "stored-private");
        assert_eq!(id.public_key, "stored-public");
    }

    #[test]
    fn half_a_pair_is_not_trusted() {
        let c = config("half");
        c.set(PRIVATE_KEY, &"only-private".to_string());

        let id = identity(&c);
        assert_ne!(id.private_key, "only-private");
        assert!(!id.public_key.is_empty());
    }

    #[test]
    fn p2p_is_on_unless_it_was_turned_off() {
        let c = config("switch");
        assert!(enabled(&c));
        c.set("ui.p2p_enabled", &false);
        assert!(!enabled(&c));
    }
}
