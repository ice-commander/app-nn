use crate::mainwindow::MainWindow;

#[cfg(feature = "nodeinnet")]
use client_core;
#[cfg(not(feature = "nodeinnet"))]
use crate::core::client_core;

pub struct Application;

impl Application {
    pub fn init_resources() {
        gtk_fm_ui::init_resources();
        gtk_sysinfo_ui::init_resources();
        gtk_terminal_ui::init_resources();
        gtk_registry_ui::init_resources();
        gtk_process_ui::init_resources();
        #[cfg(feature = "nodeinnet")]
        gtk_graph_ui::init_resources();
        #[cfg(feature = "nodeinnet")]
        node_auth::init_resources();
    }

    pub fn run(app: &adw::Application, config: client_config::AppConfig) {
        ic_utils::app::init_exe_path();

        let (ui_tx, ui_rx) = std::sync::mpsc::channel::<crate::core::UiEvent>();

        #[cfg(feature = "nodeinnet")]
        let (net_tx, net_rx) = tokio::sync::mpsc::channel::<client_core::NetCmd>(100);
        #[cfg(feature = "nodeinnet")]
        let net_tx_bg = net_tx.clone();
        #[cfg(not(feature = "nodeinnet"))]
        let net_tx = crate::core::NetCmdSender;

        #[cfg(feature = "nodeinnet")]
        let handler: std::sync::Arc<dyn client_core::AppEventHandler> =
            std::sync::Arc::new(crate::core::GtkFmEventHandler {
                ui_tx: ui_tx.clone(),
            });

        let mut needs_save = false;

        let device_id = if let Some(id) = config.get::<String>("app.device_id") {
            id
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            config.set("app.device_id", &id);
            needs_save = true;
            id
        };

        if config.get::<Vec<String>>("ui.favorites").is_none() {
            config.set(
                "ui.favorites",
                vec!["local_fs:/".to_string(), "local_fs:~".to_string()],
            );
            needs_save = true;
        }

        if needs_save {
            config.save();
        }

        #[allow(unused_mut)]
        let mut priv_key_str = String::new();
        #[allow(unused_mut)]
        let mut pub_key_str = String::new();

        #[cfg(feature = "nodeinnet")]
        {
            let identity = p2p_runtime::boot::identity(&config);
            priv_key_str = identity.private_key;
            pub_key_str = identity.public_key;
            p2p_runtime::boot::apply_api_endpoint(&config);
        }

        crate::logging::apply(&config);
        ic_logging::info!(
            "Ice Commander {} ({}) starting",
            common::version::APP_VERSION,
            common::version::BUILD_TYPE
        );

        virtualfs::set_connect_timeout_secs(config.get::<u64>("net.connect_timeout_secs").unwrap_or(20));
        virtualfs::set_request_timeout_secs(config.get::<u64>("net.request_timeout_secs").unwrap_or(20));

        let my_info = ic_model::DeviceInfo {
            id: device_id,
            name: crate::device_name::current(&config),
            os: std::env::consts::OS.to_string(),
            version: common::version::APP_VERSION.to_string(),
            app_type: "desktop".to_string(),
            build_type: common::version::BUILD_TYPE.to_string(),
        };

        let node_info = nodeinnet_p2p::NodeInfo {
            id: my_info.id.clone(),
            name: my_info.name.clone(),
            os: my_info.os.clone(),
            version: my_info.version.clone(),
            app_type: my_info.app_type.clone(),
            build_type: my_info.build_type.clone(),
            public_key: pub_key_str,
            resources: crate::shares::build_all_resources(&config, &my_info.id),
            is_online: true,
            last_used: 0,
            is_temporary: config.get::<bool>("app.is_guest").unwrap_or(false),
        };

        #[cfg(feature = "nodeinnet")]
        {
            p2p_node::set_app_version(common::version::APP_VERSION);
            p2p_handlers::install(
                p2p_handlers::Capabilities::FILESYSTEM | p2p_handlers::Capabilities::SYSTEM_INFO,
                p2p_handlers::HostSettings::default(),
            );
        }

        #[cfg(feature = "nodeinnet")]
        if p2p_runtime::boot::enabled(&config) {
            client_core::network::start_network_thread(
                net_rx,
                net_tx_bg,
                handler,
                node_info.clone(),
                priv_key_str,
                std::sync::Arc::new(client_config::ConfigPeerStore::new(config.clone())),
                config.get::<bool>("net.local_discovery").unwrap_or(true),
            );
        }

        print_startup_banner(&my_info);

        let args: Vec<String> = std::env::args().collect();
        let headless = args.iter().any(|a| a == "--headless");
        let webui = args.iter().any(|a| a == "--webui");
        let port: u16 = args.iter()
            .skip_while(|a| *a != "--port")
            .nth(1)
            .and_then(|v| v.parse().ok())
            .unwrap_or(7878);
        let host: String = args.iter()
            .skip_while(|a| *a != "--host")
            .nth(1)
            .cloned()
            .unwrap_or_else(|| "127.0.0.1".to_string());
        let network_warning: Option<(String, u16)>;
        let api_tx = if headless {
            let is_network = host != "127.0.0.1" && host != "localhost";
            if is_network {
                eprintln!("[API] WARNING: binding to {host} — REST API accessible from other machines on the network!");
            }
            network_warning = if is_network { Some((host.clone(), port)) } else { None };
            let (tx, rx) = tokio::sync::mpsc::channel::<crate::api::ApiCmd>(64);
            let ws_sessions = crate::api::WsSessions::default();
            crate::api::init_notifier(ws_sessions.clone());
            let term_out_left = tokio::sync::broadcast::channel::<Vec<u8>>(1024).0;
            let term_out_right = tokio::sync::broadcast::channel::<Vec<u8>>(1024).0;
            crate::api::start_api_server(
                port, webui, host,
                tx.clone(), ws_sessions.clone(),
                term_out_left.clone(), term_out_right.clone(),
                include_bytes!("../assets/webui/bundle.js").to_vec(),
                include_bytes!("../assets/webui/style.css").to_vec(),
            );
            Some((tx, rx, ws_sessions, term_out_left, term_out_right))
        } else {
            network_warning = None;
            None
        };

        MainWindow::create(
            app,
            my_info,
            node_info,
            net_tx,
            ui_tx,
            ui_rx,
            config.clone(),
            api_tx,
            network_warning,
        );
    }
}

fn print_startup_banner(my_info: &ic_model::DeviceInfo) {
    println!("====================================================");
    println!("🚀 IceCommander - Desktop Node Starting Up");
    println!("   Version   : v{}", my_info.version);
    println!("   OS        : {}", my_info.os);
    println!("   App Type  : {}", my_info.app_type);
    println!("   Build Type: {}", my_info.build_type);
    println!("   Node ID   : {}", my_info.id);
    println!("====================================================");
}
