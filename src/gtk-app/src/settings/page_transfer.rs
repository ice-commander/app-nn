use adw::prelude::*;
use gtk::{Align, Label};

pub(super) fn build(page_box: &gtk::Box, config: client_config::AppConfig) {
    let page_title = Label::builder()
        .label("<span size='x-large' weight='bold'>Transfer Settings</span>")
        .use_markup(true)
        .halign(Align::Start)
        .margin_bottom(16)
        .build();
    page_box.append(&page_title);

    let conn_group = adw::PreferencesGroup::builder()
        .title("Connection Settings")
        .description("Configure client connection timeouts")
        .build();

    let config_conn = config.clone();
    let entry_conn_timeout = gtk::Entry::builder()
        .text(
            &config
                .get::<u32>("ui.connection_timeout")
                .unwrap_or(30)
                .to_string(),
        )
        .valign(Align::Center)
        .width_request(100)
        .build();
    entry_conn_timeout.connect_changed(move |editable| {
        if let Ok(val) = editable.text().to_string().parse::<u32>() {
            config_conn.set("ui.connection_timeout", val);
            config_conn.save();
        }
    });

    let row_conn_timeout = adw::ActionRow::builder()
        .title("Connection Timeout (seconds)")
        .subtitle("Maximum duration to establish a connection")
        .build();
    row_conn_timeout.add_suffix(&entry_conn_timeout);
    conn_group.add(&row_conn_timeout);

    let config_req = config.clone();
    let entry_req_timeout = gtk::Entry::builder()
        .text(
            &config
                .get::<u32>("ui.request_timeout")
                .unwrap_or(60)
                .to_string(),
        )
        .valign(Align::Center)
        .width_request(100)
        .build();
    entry_req_timeout.connect_changed(move |editable| {
        if let Ok(val) = editable.text().to_string().parse::<u32>() {
            config_req.set("ui.request_timeout", val);
            config_req.save();
        }
    });

    let row_req_timeout = adw::ActionRow::builder()
        .title("Request Timeout (seconds)")
        .subtitle("Maximum duration to wait for a command response")
        .build();
    row_req_timeout.add_suffix(&entry_req_timeout);
    conn_group.add(&row_req_timeout);

    let p2p_group = adw::PreferencesGroup::builder()
        .title("Peer-to-Peer Network")
        .description("Configure settings for Node.In.Net peer network")
        .margin_top(24)
        .build();

    let p2p_enable_row = adw::SwitchRow::builder()
        .title("Enable Peer-to-Peer")
        .subtitle("Join the Node.In.Net mesh on startup. Applies after a restart.")
        .active(config.get::<bool>("ui.p2p_enabled").unwrap_or(true))
        .build();
    let config_p2p_enabled = config.clone();
    p2p_enable_row.connect_active_notify(move |row| {
        config_p2p_enabled.set("ui.p2p_enabled", row.is_active());
        config_p2p_enabled.save();
    });
    p2p_group.add(&p2p_enable_row);

    let config_peers = config.clone();
    let entry_max_peers = gtk::Entry::builder()
        .text(&config.get::<u32>("ui.max_peers").unwrap_or(10).to_string())
        .valign(Align::Center)
        .width_request(100)
        .build();
    entry_max_peers.connect_changed(move |editable| {
        if let Ok(val) = editable.text().to_string().parse::<u32>() {
            config_peers.set("ui.max_peers", val);
            config_peers.save();
        }
    });

    let row_max_peers = adw::ActionRow::builder()
        .title("Max Connected Peers")
        .subtitle("Maximum simultaneous peer connections allowed")
        .build();
    row_max_peers.add_suffix(&entry_max_peers);
    p2p_group.add(&row_max_peers);

    let config_bandwidth = config.clone();
    let entry_bandwidth = gtk::Entry::builder()
        .text(
            &config
                .get::<u32>("ui.bandwidth_limit")
                .unwrap_or(0)
                .to_string(),
        )
        .valign(Align::Center)
        .width_request(100)
        .build();
    entry_bandwidth.connect_changed(move |editable| {
        if let Ok(val) = editable.text().to_string().parse::<u32>() {
            config_bandwidth.set("ui.bandwidth_limit", val);
            config_bandwidth.save();
        }
    });

    let row_bandwidth = adw::ActionRow::builder()
        .title("Bandwidth Limit (KB/s)")
        .subtitle("Set connection limit speed (0 for unlimited)")
        .build();
    row_bandwidth.add_suffix(&entry_bandwidth);
    p2p_group.add(&row_bandwidth);

    page_box.append(&conn_group);
    page_box.append(&p2p_group);
}
