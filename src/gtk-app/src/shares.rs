use std::collections::BTreeMap;

pub const SHARES_KEY: &str = "app.shares";

pub fn get_shares(config: &client_config::AppConfig) -> BTreeMap<String, String> {
    config
        .get::<BTreeMap<String, String>>(SHARES_KEY)
        .unwrap_or_default()
}

fn save_shares(config: &client_config::AppConfig, shares: &BTreeMap<String, String>) {
    config.set(SHARES_KEY, shares);
    config.save();
}

fn sanitize_name(name: &str) -> String {
    let cleaned: String = name
        .trim()
        .chars()
        .map(|c| if c == '/' || c == '\\' { '-' } else { c })
        .collect();
    let cleaned = cleaned.trim().to_string();
    if cleaned.is_empty() {
        "Shared folder".to_string()
    } else {
        cleaned
    }
}

pub fn add_share(config: &client_config::AppConfig, name: &str, path: &str) -> String {
    let mut shares = get_shares(config);
    let base = sanitize_name(name);
    let mut unique = base.clone();
    let mut n = 2;
    while shares.contains_key(&unique) {
        unique = format!("{base} ({n})");
        n += 1;
    }
    shares.insert(unique.clone(), path.to_string());
    save_shares(config, &shares);
    unique
}

pub fn remove_share(config: &client_config::AppConfig, name: &str) {
    let mut shares = get_shares(config);
    shares.remove(name);
    save_shares(config, &shares);
}

pub fn set_share_path(config: &client_config::AppConfig, name: &str, path: &str) {
    let mut shares = get_shares(config);
    if let Some(v) = shares.get_mut(name) {
        *v = path.to_string();
        save_shares(config, &shares);
    }
}

pub fn fs_resource_id(device_id: &str) -> String {
    uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_DNS,
        format!("{device_id}-fs").as_bytes(),
    )
    .to_string()
}

pub fn sysinfo_resource_id(device_id: &str) -> String {
    uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_DNS,
        format!("{device_id}-sysinfo").as_bytes(),
    )
    .to_string()
}

pub fn build_all_resources(
    config: &client_config::AppConfig,
    device_id: &str,
) -> Vec<nodeinnet_p2p::SharedResource> {
    let mut resources = vec![nodeinnet_p2p::SharedResource {
        id: sysinfo_resource_id(device_id),
        name: "System Info".to_string(),
        resource_type: nodeinnet_p2p::ResourceType::SystemInfo,
        config: None,
        is_active: true,
        session_token: None,
    }];

    let shares = get_shares(config);
    if !shares.is_empty() {
        let name = if shares.len() == 1 {
            shares.keys().next().cloned().unwrap_or_default()
        } else {
            crate::i18n::tr("shares.files").to_string()
        };
        let share_list: Vec<serde_json::Value> = shares
            .iter()
            .map(|(name, path)| serde_json::json!({ "name": name, "path": path }))
            .collect();
        resources.push(nodeinnet_p2p::SharedResource {
            id: fs_resource_id(device_id),
            name,
            resource_type: nodeinnet_p2p::ResourceType::Filesystem,
            config: Some(serde_json::json!({ "shares": share_list }).to_string()),
            is_active: true,
            session_token: None,
        });
    }

    resources
}

pub fn reload_resources(
    config: &client_config::AppConfig,
    device_id: &str,
    net_tx: &crate::core::NetCmdSender,
) {
    #[cfg(feature = "nodeinnet")]
    use client_core;
    #[cfg(not(feature = "nodeinnet"))]
    use crate::core::client_core;

    let _ = net_tx.try_send(client_core::NetCmd::ReloadResources(build_all_resources(
        config, device_id,
    )));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> client_config::AppConfig {
        let c = client_config::AppConfig::new("_ice_commander_unit_tests_shares");
        c.set(SHARES_KEY, BTreeMap::<String, String>::new());
        c
    }

    #[test]
    fn add_share_dedups_names() {
        let c = cfg();
        assert_eq!(add_share(&c, "Docs", "/a"), "Docs");
        assert_eq!(add_share(&c, "Docs", "/b"), "Docs (2)");
        assert_eq!(add_share(&c, "Docs", "/c"), "Docs (3)");
        let shares = get_shares(&c);
        assert_eq!(shares.len(), 3);
        assert_eq!(shares["Docs (2)"], "/b");
    }

    #[test]
    fn remove_and_repath() {
        let c = cfg();
        add_share(&c, "Music", "/m");
        set_share_path(&c, "Music", "/m2");
        assert_eq!(get_shares(&c)["Music"], "/m2");
        remove_share(&c, "Music");
        assert!(get_shares(&c).is_empty());
    }

    #[test]
    fn single_share_resource_carries_its_name_and_shares_json() {
        let c = cfg();
        add_share(&c, "Photos", "/p");
        let res = build_all_resources(&c, "dev-1");
        assert_eq!(res.len(), 2); // SystemInfo + Filesystem
        let fs = &res[1];
        assert_eq!(fs.resource_type, nodeinnet_p2p::ResourceType::Filesystem);
        assert_eq!(fs.name, "Photos");
        assert_eq!(fs.id, fs_resource_id("dev-1"));
        let cfg_json: serde_json::Value =
            serde_json::from_str(fs.config.as_ref().unwrap()).unwrap();
        assert_eq!(cfg_json["shares"][0]["name"], "Photos");
        assert_eq!(cfg_json["shares"][0]["path"], "/p");
    }

    #[test]
    fn no_shares_means_no_fs_resource() {
        let c = cfg();
        let res = build_all_resources(&c, "dev-1");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].resource_type, nodeinnet_p2p::ResourceType::SystemInfo);
    }

    #[test]
    fn two_shares_get_umbrella_name_and_both_in_json() {
        let c = cfg();
        add_share(&c, "Docs", "/docs");
        add_share(&c, "Music", "/music");
        let res = build_all_resources(&c, "dev-1");
        let fs = &res[1];
        assert_ne!(fs.name, "Docs");
        assert_ne!(fs.name, "Music");
        let cfg_json: serde_json::Value =
            serde_json::from_str(fs.config.as_ref().unwrap()).unwrap();
        let arr = cfg_json["shares"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], "Docs");
        assert_eq!(arr[0]["path"], "/docs");
        assert_eq!(arr[1]["name"], "Music");
        assert_eq!(arr[1]["path"], "/music");
    }

    #[test]
    fn share_name_separators_are_folded() {
        let c = cfg();
        assert_eq!(add_share(&c, "My/Secret\\Docs", "/x"), "My-Secret-Docs");
        assert_eq!(add_share(&c, "  ", "/y"), "Shared folder"); // blank → default
    }

    #[test]
    fn fs_resource_id_is_stable() {
        assert_eq!(fs_resource_id("dev-1"), fs_resource_id("dev-1"));
        assert_ne!(fs_resource_id("dev-1"), fs_resource_id("dev-2"));
    }

    #[cfg(feature = "nodeinnet")]
    #[test]
    fn config_json_matches_p2p_node_router_type() {
        use p2p_handlers::fs_local::FsConfig;

        let parse = |res: &nodeinnet_p2p::SharedResource| -> FsConfig {
            serde_json::from_str::<FsConfig>(res.config.as_ref().unwrap()).unwrap()
        };

        let c = cfg();
        add_share(&c, "Photos", "/home/me/pics");
        let one = build_all_resources(&c, "dev-1");
        let fs_one = parse(&one[1]);
        assert_eq!(fs_one.shares.len(), 1);
        assert_eq!(fs_one.shares[0].name, "Photos");
        assert_eq!(fs_one.shares[0].path, "/home/me/pics");

        add_share(&c, "Videos", "/home/me/vids");
        let many = build_all_resources(&c, "dev-1");
        let fs_many = parse(&many[1]);
        let names: Vec<&str> = fs_many.shares.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Photos") && names.contains(&"Videos"));
        assert_eq!(fs_many.shares.len(), 2);
    }
}
