use adw::prelude::*;
use gtk::glib;
use relm4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

pub type UpdateGraphFn = Box<dyn Fn(&[nodeinnet_p2p::NodeInfo])>;

pub struct LoginInit {
    pub logo_resource: String,
    pub prefix: String,
    pub my_node_id: String,
    pub app_version: String,
    pub config: client_config::AppConfig,
    pub graph_widget: Option<gtk::Widget>,
    pub update_graph: Option<UpdateGraphFn>,
}

#[derive(Debug)]
pub enum LoginInput {
    Submit {
        login: String,
        password: String,
        device_name: String,
        guest: bool,
    },
    LoginFailed(String),
    LoginSucceeded,
    PeersChanged(Vec<nodeinnet_p2p::NodeInfo>),
    Reset,
    #[cfg(feature = "screen-capture")]
    InstallRequested,
    #[cfg(feature = "screen-capture")]
    InstallFinished(Result<(), String>),
    #[cfg(feature = "screen-capture")]
    HideInstallBanner,
}

#[derive(Debug)]
pub enum LoginOutput {
    SubmitLogin {
        login: String,
        password: String,
        device_name: String,
        guest: bool,
    },
    TransitionFinished,
}

#[derive(PartialEq)]
enum Phase {
    Form,
    Transition,
}

#[cfg(feature = "screen-capture")]
#[derive(Clone, PartialEq)]
enum InstallState {
    Hidden,
    Offered,
    Installing,
    Succeeded,
    Failed(String),
}

fn device_name_key(prefix: &str) -> &'static str {
    match prefix {
        "fm" => "app-fm-name",
        "net" => "app-net-name",
        "sync" => "app-sync-name",
        "rdesk" => "app-rdesk-name",
        _ => "app-name",
    }
}

pub struct LoginModel {
    prefix: String,
    my_node_id: String,
    config: client_config::AppConfig,
    update_graph: Option<UpdateGraphFn>,
    busy: bool,
    error: Option<String>,
    phase: Phase,
    pending_device_name: String,
    peers: Vec<nodeinnet_p2p::NodeInfo>,
    peers_generation: u64,
    reset_generation: u64,
    transition_gate: Rc<Cell<bool>>,
    #[cfg(feature = "screen-capture")]
    install_state: InstallState,
}

pub struct LoginWidgets {
    stack: gtk::Stack,
    login_btn: gtk::Button,
    error_label: gtk::Label,
    pass_entry: gtk::PasswordEntry,
    peer_status_lbl: gtk::Label,
    #[cfg(feature = "screen-capture")]
    banner: adw::Banner,
    rendered_peers_generation: u64,
    rendered_reset_generation: u64,
}

impl SimpleComponent for LoginModel {
    type Init = LoginInit;
    type Input = LoginInput;
    type Output = LoginOutput;
    type Root = gtk::Stack;
    type Widgets = LoginWidgets;

    fn init_root() -> Self::Root {
        gtk::Stack::builder()
            .transition_type(gtk::StackTransitionType::SlideLeftRight)
            .transition_duration(500)
            .build()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        crate::init_resources();
        let config = init.config;

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 16);
        vbox.set_margin_start(32);
        vbox.set_margin_end(32);
        vbox.set_margin_top(32);
        vbox.set_margin_bottom(32);
        vbox.set_valign(gtk::Align::Center);
        vbox.set_halign(gtk::Align::Center);
        vbox.set_width_request(340);

        let brand_image = gtk::Image::from_resource(&init.logo_resource);
        brand_image.set_pixel_size(256);
        brand_image.set_halign(gtk::Align::Center);
        brand_image.set_margin_bottom(4);

        let version_lbl = gtk::Label::new(Some(&format!("v{}", init.app_version)));
        version_lbl.add_css_class("dim-label");
        version_lbl.set_halign(gtk::Align::Center);
        version_lbl.set_margin_bottom(16);

        let header_hbox = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        header_hbox.set_halign(gtk::Align::Center);
        header_hbox.set_valign(gtk::Align::Center);
        header_hbox.set_margin_bottom(16);
        header_hbox.set_margin_top(16);

        let login_icon = gtk::Image::from_resource("/com/node-auth/gtk/login.svg");
        login_icon.set_pixel_size(80);
        login_icon.set_valign(gtk::Align::Center);

        let text_vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
        text_vbox.set_valign(gtk::Align::Center);

        let label = gtk::Label::new(Some(&*crate::i18n::tr("auth.production_login")));
        label.add_css_class("title-1");
        label.set_halign(gtk::Align::Start);

        let subtitle = gtk::Label::new(Some(&*crate::i18n::tr("auth.signin_prompt")));
        subtitle.add_css_class("dim-label");
        subtitle.set_halign(gtk::Align::Start);
        subtitle.set_wrap(true);

        text_vbox.append(&label);
        text_vbox.append(&subtitle);

        header_hbox.append(&login_icon);
        header_hbox.append(&text_vbox);

        let login_entry = gtk::Entry::builder()
            .placeholder_text(&*crate::i18n::tr("auth.login_placeholder"))
            .build();
        let pass_entry = gtk::PasswordEntry::builder()
            .placeholder_text(&*crate::i18n::tr("auth.password_placeholder"))
            .show_peek_icon(true)
            .build();

        let device_name_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let device_name_lbl = gtk::Label::builder()
            .label(&*crate::i18n::tr("auth.device_name"))
            .halign(gtk::Align::Start)
            .build();
        device_name_lbl.add_css_class("caption");

        let host_name = hostname::get()
            .ok()
            .and_then(|s| s.into_string().ok())
            .unwrap_or_else(|| "Node".to_string());

        let default_device_name = config
            .get::<String>(device_name_key(&init.prefix))
            .or_else(|| config.get::<String>("app.device_name"))
            .unwrap_or_else(|| {
                if init.prefix.is_empty() {
                    host_name
                } else {
                    format!("{}-{}", init.prefix, host_name)
                }
            });

        let device_name_entry = gtk::Entry::builder()
            .placeholder_text(&*crate::i18n::tr("auth.device_name"))
            .text(&default_device_name)
            .build();

        let device_name_sub = gtk::Label::builder()
            .label(&*crate::i18n::tr("auth.device_name_help"))
            .halign(gtk::Align::Start)
            .build();
        device_name_sub.add_css_class("dim-label");
        device_name_sub.add_css_class("caption");

        device_name_box.append(&device_name_lbl);
        device_name_box.append(&device_name_entry);
        device_name_box.append(&device_name_sub);

        let guest_check = gtk::CheckButton::builder()
            .label(&*crate::i18n::tr("auth.login_as_guest"))
            .active(false)
            .build();

        let error_label = gtk::Label::builder().label("").visible(false).build();
        error_label.add_css_class("error");

        let btn_login = gtk::Button::builder()
            .label(&*crate::i18n::tr("auth.secure_login"))
            .css_classes(vec!["suggested-action", "pill"])
            .build();
        btn_login.set_cursor_from_name(Some("pointer"));

        #[cfg(feature = "screen-capture")]
        let (banner, initial_install_state) = {
            let banner = adw::Banner::new(&crate::i18n::tr("auth.dependencies_missing"));
            banner.set_button_label(Some(&crate::i18n::tr("auth.install")));
            banner.set_revealed(false);

            let gst_ok = node_functions::desktop::is_gstreamer_pipewire_available();
            let portal_ok = node_functions::desktop::is_xdg_desktop_portal_installed();
            let state = if cfg!(target_os = "linux") && (!gst_ok || !portal_ok) {
                InstallState::Offered
            } else {
                InstallState::Hidden
            };

            let sender_install = sender.clone();
            banner.connect_button_clicked(move |_| {
                sender_install.input(LoginInput::InstallRequested);
            });
            (banner, state)
        };

        vbox.append(&brand_image);
        vbox.append(&version_lbl);
        vbox.append(&header_hbox);
        vbox.append(&login_entry);
        vbox.append(&pass_entry);
        vbox.append(&device_name_box);
        vbox.append(&guest_check);
        vbox.append(&error_label);
        vbox.append(&btn_login);
        #[cfg(feature = "screen-capture")]
        vbox.append(&banner);

        root.add_named(&vbox, Some("login"));

        let trans_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .width_request(350)
            .margin_top(24)
            .margin_bottom(24)
            .margin_start(24)
            .margin_end(24)
            .build();

        let success_lbl = gtk::Label::new(None);
        success_lbl.set_markup(&format!(
            "<span size='large' weight='bold' foreground='#00cc66'>{}</span>",
            crate::i18n::tr("auth.login_successful")
        ));
        success_lbl.set_halign(gtk::Align::Center);
        trans_box.append(&success_lbl);

        let connect_lbl = gtk::Label::new(Some(&*crate::i18n::tr("auth.connecting")));
        connect_lbl.add_css_class("dim-label");
        connect_lbl.set_halign(gtk::Align::Center);
        trans_box.append(&connect_lbl);

        if let Some(ref graph_widget) = init.graph_widget {
            trans_box.append(graph_widget);
        }

        let peer_status_lbl = gtk::Label::new(None);
        peer_status_lbl.set_halign(gtk::Align::Center);
        peer_status_lbl.set_wrap(true);
        peer_status_lbl.set_justify(gtk::Justification::Center);
        trans_box.append(&peer_status_lbl);

        root.add_named(&trans_box, Some("transition"));
        root.set_visible_child_name("login");

        let submit = {
            let login_entry = login_entry.clone();
            let pass_entry = pass_entry.clone();
            let device_name_entry = device_name_entry.clone();
            let guest_check = guest_check.clone();
            let sender = sender.clone();
            Rc::new(move || {
                sender.input(LoginInput::Submit {
                    login: login_entry.text().to_string(),
                    password: pass_entry.text().to_string(),
                    device_name: device_name_entry.text().to_string(),
                    guest: guest_check.is_active(),
                });
            })
        };
        let submit_click = submit.clone();
        btn_login.connect_clicked(move |_| submit_click());
        let submit_enter = submit.clone();
        pass_entry.connect_activate(move |_| submit_enter());

        let model = LoginModel {
            prefix: init.prefix,
            my_node_id: init.my_node_id,
            config,
            update_graph: init.update_graph,
            busy: false,
            error: None,
            phase: Phase::Form,
            pending_device_name: String::new(),
            peers: Vec::new(),
            peers_generation: 0,
            reset_generation: 0,
            transition_gate: Rc::new(Cell::new(false)),
            #[cfg(feature = "screen-capture")]
            install_state: initial_install_state,
        };
        let widgets = LoginWidgets {
            stack: root,
            login_btn: btn_login,
            error_label,
            pass_entry,
            peer_status_lbl,
            #[cfg(feature = "screen-capture")]
            banner,
            rendered_peers_generation: 0,
            rendered_reset_generation: 0,
        };
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            LoginInput::Submit {
                login,
                password,
                device_name,
                guest,
            } => {
                self.pending_device_name = device_name.clone();
                self.busy = true;
                self.error = None;
                let _ = sender.output(LoginOutput::SubmitLogin {
                    login,
                    password,
                    device_name,
                    guest,
                });
            }
            LoginInput::LoginFailed(msg) => {
                self.busy = false;
                self.error = Some(msg);
            }
            LoginInput::LoginSucceeded => {
                self.config.set(
                    device_name_key(&self.prefix),
                    self.pending_device_name.clone(),
                );
                self.config.save();

                self.busy = false;
                self.error = None;
                self.phase = Phase::Transition;

                self.transition_gate.set(true);
                let gate = self.transition_gate.clone();
                let sender_done = sender.clone();
                glib::timeout_add_local(std::time::Duration::from_secs(5), move || {
                    if gate.get() {
                        gate.set(false);
                        let _ = sender_done.output(LoginOutput::TransitionFinished);
                    }
                    glib::ControlFlow::Break
                });
            }
            LoginInput::PeersChanged(peers) => {
                self.peers = peers;
                self.peers_generation += 1;
            }
            LoginInput::Reset => {
                self.phase = Phase::Form;
                self.busy = false;
                self.error = None;
                self.transition_gate.set(false);
                self.reset_generation += 1;
            }
            #[cfg(feature = "screen-capture")]
            LoginInput::InstallRequested => {
                if self.install_state == InstallState::Installing {
                    return;
                }
                self.install_state = InstallState::Installing;
                let sender = sender.clone();
                std::thread::spawn(move || {
                    if let Ok(rt) = tokio::runtime::Runtime::new() {
                        rt.block_on(async {
                            let gst_res =
                                if !node_functions::desktop::is_gstreamer_pipewire_available() {
                                    node_functions::desktop::run_gstreamer_installer().await
                                } else {
                                    Ok(())
                                };

                            let portal_res =
                                if !node_functions::desktop::is_xdg_desktop_portal_installed() {
                                    node_functions::desktop::run_portal_installer().await
                                } else {
                                    Ok(())
                                };

                            let res = if gst_res.is_ok() && portal_res.is_ok() {
                                node_functions::desktop::update_gstreamer_registry();
                                Ok(())
                            } else {
                                Err(format!(
                                    "GStreamer: {:?}, Portal: {:?}",
                                    gst_res.err().unwrap_or_else(|| "OK".to_string()),
                                    portal_res.err().unwrap_or_else(|| "OK".to_string())
                                ))
                            };
                            sender.input(LoginInput::InstallFinished(res));
                        });
                    }
                });
            }
            #[cfg(feature = "screen-capture")]
            LoginInput::InstallFinished(res) => {
                self.install_state = match res {
                    Ok(()) => InstallState::Succeeded,
                    Err(e) => InstallState::Failed(e),
                };
                if self.install_state == InstallState::Succeeded {
                    let sender_hide = sender.clone();
                    glib::timeout_add_local(std::time::Duration::from_secs(3), move || {
                        sender_hide.input(LoginInput::HideInstallBanner);
                        glib::ControlFlow::Break
                    });
                }
            }
            #[cfg(feature = "screen-capture")]
            LoginInput::HideInstallBanner => {
                self.install_state = InstallState::Hidden;
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        widgets.login_btn.set_sensitive(!self.busy);

        match &self.error {
            Some(msg) => {
                widgets.error_label.set_label(msg);
                widgets.error_label.set_visible(true);
            }
            None => widgets.error_label.set_visible(false),
        }

        widgets.stack.set_visible_child_name(match self.phase {
            Phase::Form => "login",
            Phase::Transition => "transition",
        });

        if widgets.rendered_reset_generation != self.reset_generation {
            widgets.rendered_reset_generation = self.reset_generation;
            widgets.pass_entry.set_text("");
        }

        if widgets.rendered_peers_generation != self.peers_generation {
            widgets.rendered_peers_generation = self.peers_generation;
            if let Some(ref update_graph) = self.update_graph {
                update_graph(&self.peers);
            }
            let other_peers_count = self
                .peers
                .iter()
                .filter(|n| n.id != self.my_node_id)
                .count();
            if other_peers_count == 0 {
                widgets.peer_status_lbl.set_markup(&format!(
                    "<span foreground='#ff9900' weight='bold'>{}</span>\n<span size='small' alpha='70%'>{}</span>",
                    crate::i18n::tr("auth.no_devices"),
                    crate::i18n::tr("auth.no_devices_help")
                ));
            } else {
                widgets.peer_status_lbl.set_markup(&format!(
                    "<span foreground='#00cc66' weight='bold'>{}</span>",
                    crate::i18n::trf(
                        "auth.connected_devices",
                        &[("count", &*(other_peers_count).to_string())]
                    )
                ));
            }
        }

        #[cfg(feature = "screen-capture")]
        {
            match &self.install_state {
                InstallState::Hidden => widgets.banner.set_revealed(false),
                InstallState::Offered => {
                    widgets
                        .banner
                        .set_title(&crate::i18n::tr("auth.dependencies_missing"));
                    widgets
                        .banner
                        .set_button_label(Some(&crate::i18n::tr("auth.install")));
                    widgets.banner.set_sensitive(true);
                    widgets.banner.set_revealed(true);
                }
                InstallState::Installing => {
                    widgets
                        .banner
                        .set_title(&crate::i18n::tr("auth.installing_requirements"));
                    widgets.banner.set_sensitive(false);
                    widgets.banner.set_revealed(true);
                }
                InstallState::Succeeded => {
                    widgets
                        .banner
                        .set_title(&crate::i18n::tr("auth.install_success"));
                    widgets.banner.set_button_label(None);
                    widgets.banner.set_sensitive(true);
                    widgets.banner.set_revealed(true);
                }
                InstallState::Failed(e) => {
                    widgets.banner.set_title(&crate::i18n::trf(
                        "auth.install_failed",
                        &[("error", &*(e).to_string())],
                    ));
                    widgets.banner.set_sensitive(true);
                    widgets.banner.set_revealed(true);
                }
            }
        }
    }
}
