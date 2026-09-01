use adw::prelude::*;
use nodeinnet_p2p::NodeInfo;
use relm4::prelude::*;

use crate::graph_view::{
    NetworkGraphInit, NetworkGraphInput, NetworkGraphModel, NetworkGraphOutput,
};

pub struct WelcomePanelInit {
    pub title: String,
    pub name_key: String,
    pub name_prefix: String,
    pub config: client_config::AppConfig,
    pub extra_widget: Option<gtk::Widget>,
    pub graph: NetworkGraphInit,
}

#[derive(Debug)]
pub enum WelcomePanelInput {
    SetAccount { login: String, premium: bool },
    SetDeviceName(String),
    UpdatePeers { peers: Vec<NodeInfo>, my_id: String },
    ApplyPressed(String),
}

#[derive(Debug)]
pub enum WelcomePanelOutput {
    ApplyDeviceName(String),
    NodeClicked(String),
}

pub struct WelcomePanelModel {
    title: String,
    name_key: String,
    config: client_config::AppConfig,
    login: Option<String>,
    premium: bool,
    device_name: String,
    name_generation: u64,
    graph: Controller<NetworkGraphModel>,
}

pub struct WelcomePanelWidgets {
    welcome_label: gtk::Label,
    login_label: gtk::Label,
    pro_badge: gtk::Label,
    name_entry: gtk::Entry,
    rendered_name_generation: u64,
}

impl SimpleComponent for WelcomePanelModel {
    type Init = WelcomePanelInit;
    type Input = WelcomePanelInput;
    type Output = WelcomePanelOutput;
    type Root = gtk::Box;
    type Widgets = WelcomePanelWidgets;

    fn init_root() -> Self::Root {
        gtk::Box::new(gtk::Orientation::Vertical, 0)
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        crate::init_resources();
        crate::init_styles();
        let config = init.config;

        let host_name = hostname::get()
            .ok()
            .and_then(|s| s.into_string().ok())
            .unwrap_or_else(|| "Node".to_string());
        let mut name_needs_save = false;
        let device_name = if let Some(name) = config.get::<String>(&init.name_key) {
            name
        } else if let Some(name) = config.get::<String>("app.device_name") {
            config.set(&init.name_key, &name);
            name_needs_save = true;
            name
        } else {
            let name = if init.name_prefix.is_empty() {
                host_name
            } else {
                format!("{}-{}", init.name_prefix, host_name)
            };
            config.set(&init.name_key, &name);
            name_needs_save = true;
            name
        };
        if name_needs_save {
            config.save();
        }

        let graph = NetworkGraphModel::builder().launch(init.graph).forward(
            sender.output_sender(),
            |msg| match msg {
                NetworkGraphOutput::NodeClicked(id) => WelcomePanelOutput::NodeClicked(id),
            },
        );

        let welcome_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .spacing(16)
            .build();

        let welcome_label = gtk::Label::builder()
            .label(&*crate::i18n::trf(
                "graph.welcome",
                &[("title", &*(init.title.as_str()).to_string())],
            ))
            .css_classes(vec!["title-1"])
            .build();
        welcome_box.append(&welcome_label);

        let login_label = gtk::Label::builder()
            .label("")
            .css_classes(vec!["title-3"])
            .build();
        let pro_badge = gtk::Label::builder()
            .label("PRO")
            .css_classes(vec!["accent"])
            .visible(false)
            .build();
        let login_hbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk::Align::Center)
            .build();
        login_hbox.append(&login_label);
        login_hbox.append(&pro_badge);
        welcome_box.append(&login_hbox);

        let name_edit_hbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk::Align::Center)
            .build();
        let name_edit_lbl = gtk::Label::builder()
            .label(&*crate::i18n::tr("graph.device_name_label"))
            .css_classes(vec!["dim-label"])
            .build();
        let name_entry = gtk::Entry::builder()
            .placeholder_text(&*crate::i18n::tr("graph.device_name_placeholder"))
            .width_request(180)
            .build();
        name_entry.set_text(&device_name);
        let name_edit_btn = gtk::Button::builder()
            .label(&*crate::i18n::tr("graph.apply_btn"))
            .css_classes(vec!["pill-button"])
            .build();
        name_edit_btn.set_cursor_from_name(Some("pointer"));
        name_edit_hbox.append(&name_edit_lbl);
        name_edit_hbox.append(&name_entry);
        name_edit_hbox.append(&name_edit_btn);
        welcome_box.append(&name_edit_hbox);

        let entry_apply = name_entry.clone();
        let sender_apply = sender.clone();
        name_edit_btn.connect_clicked(move |_| {
            sender_apply.input(WelcomePanelInput::ApplyPressed(
                entry_apply.text().to_string(),
            ));
        });

        let graph_container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .vexpand(true)
            .hexpand(true)
            .build();
        graph_container.append(graph.widget());
        welcome_box.append(&graph_container);

        if let Some(extra) = init.extra_widget {
            welcome_box.append(&extra);
        }

        let scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .child(&welcome_box)
            .vexpand(true)
            .build();
        root.append(&scroll);

        let model = WelcomePanelModel {
            title: init.title,
            name_key: init.name_key,
            config,
            login: None,
            premium: false,
            device_name,
            name_generation: 0,
            graph,
        };
        let widgets = WelcomePanelWidgets {
            welcome_label,
            login_label,
            pro_badge,
            name_entry,
            rendered_name_generation: 0,
        };
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            WelcomePanelInput::SetAccount { login, premium } => {
                self.login = Some(login);
                self.premium = premium;
            }
            WelcomePanelInput::SetDeviceName(name) => {
                self.device_name = name;
                self.name_generation += 1;
            }
            WelcomePanelInput::UpdatePeers { peers, my_id } => {
                self.graph
                    .emit(NetworkGraphInput::UpdatePeers { peers, my_id });
            }
            WelcomePanelInput::ApplyPressed(name) => {
                if name.is_empty() {
                    return;
                }
                self.device_name = name.clone();
                self.name_generation += 1;
                self.config.set(&self.name_key, &name);
                self.config.save();
                let _ = sender.output(WelcomePanelOutput::ApplyDeviceName(name));
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        widgets.welcome_label.set_label(&crate::i18n::trf(
            "graph.welcome",
            &[("title", &*(self.title.as_str()).to_string())],
        ));

        match &self.login {
            Some(login) => widgets.login_label.set_label(&crate::i18n::trf(
                "graph.logged_in_as",
                &[("user", &*(login.as_str()).to_string())],
            )),
            None => widgets.login_label.set_label(""),
        }
        widgets.pro_badge.set_visible(self.premium);

        if widgets.rendered_name_generation != self.name_generation {
            widgets.rendered_name_generation = self.name_generation;
            widgets.name_entry.set_text(&self.device_name);
        }
    }
}
