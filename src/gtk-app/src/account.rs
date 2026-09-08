use adw::prelude::*;
use gtk::glib;
use gtk::{Align, Box, Button, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(feature = "nodeinnet")]
use client_core;
#[cfg(not(feature = "nodeinnet"))]
use crate::core::client_core;


pub fn create_account_widget(
    config: client_config::AppConfig,
    ws_state: &Rc<RefCell<client_core::WsState>>,
    active_dialog_graph: &Rc<RefCell<Option<crate::netgraph::NetGraph>>>,
    online_nodes: &Rc<RefCell<Vec<nodeinnet_p2p::NodeInfo>>>,
    my_info: &nodeinnet_p2p::NodeInfo,
    net_tx: &crate::core::NetCmdSender,
    login_btn_label: &Label,
    close_callback: Rc<dyn Fn()>,
    on_resources_changed: Option<Rc<dyn Fn()>>,
) -> gtk::Widget {
    let api_base = nodeinnet_p2p::api_base();
    let net_tx_dialog = net_tx.clone();
    let my_info_dialog = my_info.clone();
    let login_lbl_dialog = login_btn_label.clone();
    let ws_state_dialog = ws_state.clone();
    let active_dialog_graph_dialog = active_dialog_graph.clone();
    let online_nodes_dialog = online_nodes.clone();

    if *ws_state_dialog.borrow() == client_core::WsState::Connected {
        let active_dialog_graph_clone = active_dialog_graph_dialog.clone();
        let net_tx_inner = net_tx_dialog.clone();
        let my_info_inner = my_info_dialog.clone();
        let online_nodes_inner = online_nodes_dialog.clone();

        let root_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_top(18)
            .margin_bottom(18)
            .margin_start(18)
            .margin_end(18)
            .build();

        let profile_hbox = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(16)
            .halign(Align::Center)
            .build();

        let avatar = gtk::Image::from_resource("/com/icecommander/gtk/login.svg");
        avatar.set_pixel_size(80);
        avatar.set_valign(Align::Center);
        profile_hbox.append(&avatar);

        let details_vbox = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .valign(Align::Center)
            .halign(Align::Center)
            .hexpand(true)
            .build();

        let status_lbl = Label::new(None);
        status_lbl.set_markup(&*crate::i18n::tr("account.authorized_status"));
        status_lbl.add_css_class("dim-label");
        status_lbl.set_halign(Align::Center);
        details_vbox.append(&status_lbl);

        let login_row = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .valign(Align::Center)
            .halign(Align::Center)
            .build();

        let username = config.get::<String>("app.account_login")
            .unwrap_or_else(|| "Unknown".to_string());
        let username_lbl = Label::new(None);
        username_lbl.set_markup(&format!(
            "<span size='large' weight='bold'>{}</span>",
            username
        ));
        username_lbl.set_halign(Align::Center);
        login_row.append(&username_lbl);

        let is_premium = config.get::<bool>("app.premium").unwrap_or(false);
        if is_premium {
            let premium_icon =
                gtk::Image::from_resource("/com/icecommander/gtk/membership-card.svg");
            premium_icon.set_pixel_size(24);
            premium_icon.set_valign(Align::Center);
            login_row.append(&premium_icon);

            let premium_lbl = Label::new(None);
            premium_lbl.set_markup(&format!(
                "<span foreground='#ff7300ff' weight='bold'>{}</span>",
                crate::i18n::tr("account.premium")
            ));
            premium_lbl.set_valign(Align::Center);
            login_row.append(&premium_lbl);
        }

        let logout_btn = Button::builder()
            .label(&*crate::i18n::tr("account.log_out"))
            .css_classes(["destructive-action"])
            .valign(Align::Center)
            .build();
        logout_btn.set_cursor_from_name(Some("pointer"));

        let active_dialog_graph_logout = active_dialog_graph_clone.clone();
        let close_cb = close_callback.clone();
        let config_logout = config.clone();
        logout_btn.connect_clicked(move |_| {
            config_logout.set("app.refresh_token", Option::<String>::None);
            config_logout.set("app.account_login", Option::<String>::None);
            config_logout.set("app.is_guest", false);
            config_logout.set("app.premium", false);
            config_logout.save();

            let net_tx_logout = net_tx_inner.clone();
            glib::spawn_future_local(async move {
                let _ = net_tx_logout.send(client_core::NetCmd::Disconnect).await;
            });

            *active_dialog_graph_logout.borrow_mut() = None;
            close_cb();
        });
        login_row.append(&logout_btn);

        details_vbox.append(&login_row);
        profile_hbox.append(&details_vbox);
        root_box.append(&profile_hbox);

        let name_section = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .halign(Align::Center)
            .build();
        let name_title = Label::new(None);
        name_title.set_markup(&format!(
            "<b>{}</b>",
            crate::i18n::tr("settings.device_name")
        ));
        name_title.set_halign(Align::Center);
        let name_entry = gtk::Entry::builder()
            .text(&crate::device_name::current(&config))
            .width_request(280)
            .halign(Align::Center)
            .build();
        name_entry.set_tooltip_text(Some(&crate::i18n::tr("settings.desc_device_name")));

        let apply_name: Rc<dyn Fn(&gtk::Entry)> = {
            let config = config.clone();
            let net_tx = net_tx_dialog.clone();
            Rc::new(move |entry: &gtk::Entry| {
                let new_name = entry.text().trim().to_string();
                if new_name.is_empty() || new_name == crate::device_name::current(&config) {
                    return;
                }
                crate::device_name::set(&config, &new_name);
                let _ = net_tx.try_send(client_core::NetCmd::UpdateName(new_name));
            })
        };
        {
            let apply = apply_name.clone();
            name_entry.connect_activate(move |e| apply(e));
        }
        {
            let apply = apply_name.clone();
            let entry = name_entry.clone();
            let focus = gtk::EventControllerFocus::new();
            focus.connect_leave(move |_| apply(&entry));
            name_entry.add_controller(focus);
        }
        name_section.append(&name_title);
        name_section.append(&name_entry);
        root_box.append(&name_section);

        let separator = gtk::Separator::new(Orientation::Horizontal);
        root_box.append(&separator);

        let graph_widget = crate::netgraph::NetGraph::new(
            gtk_graph_ui::NetworkGraphInit {
                icon_size: 40,
                interactive: false,
                show_names: true,
                show_local_node: true,
            },
            None,
        );
        graph_widget.root.set_size_request(350, 250);
        graph_widget.root.set_vexpand(true);
        graph_widget.root.set_hexpand(true);

        graph_widget.update_peers(&online_nodes_inner.borrow(), &my_info_inner.id);
        root_box.append(&graph_widget.root);
        *active_dialog_graph_clone.borrow_mut() = Some(graph_widget);

        let share_separator = gtk::Separator::new(Orientation::Horizontal);
        root_box.append(&share_separator);

        let share_header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .margin_bottom(6)
            .build();

        let share_title = Label::builder()
            .label(&format!("<b>{}</b>", crate::i18n::tr("account.shared_folders")))
            .use_markup(true)
            .halign(Align::Start)
            .hexpand(true)
            .build();
        share_header.append(&share_title);

        let add_share_box = Box::new(Orientation::Horizontal, 6);
        let add_share_img = gtk::Image::from_resource("/com/icecommander/gtk/p2p-share.svg");
        add_share_img.set_pixel_size(20);
        add_share_box.append(&add_share_img);
        add_share_box.append(&Label::new(Some(&*crate::i18n::tr("account.share_new_folder"))));

        let add_share_btn = Button::builder()
            .child(&add_share_box)
            .css_classes(["flat"])
            .tooltip_text(&*crate::i18n::tr("account.share_new_folder"))
            .build();
        add_share_btn.set_cursor_from_name(Some("pointer"));
        share_header.append(&add_share_btn);
        root_box.append(&share_header);

        let shares_list = gtk::ListBox::builder()
            .css_classes(["boxed-list"])
            .selection_mode(gtk::SelectionMode::None)
            .build();

        let shares_scrolled = gtk::ScrolledWindow::builder()
            .min_content_height(100)
            .propagate_natural_height(true)
            .child(&shares_list)
            .build();
        root_box.append(&shares_scrolled);

        let populate_shares_rc: Rc<RefCell<Option<std::boxed::Box<dyn Fn()>>>> =
            Rc::new(RefCell::new(None));
        let populate_shares = {
            let shares_list = shares_list.clone();
            let net_tx_dialog = net_tx_dialog.clone();
            let my_info_dialog = my_info_dialog.clone();
            let on_res_changed = on_resources_changed.clone();
            let populate_shares_weak = Rc::downgrade(&populate_shares_rc);
            let shares_list_root = shares_list.clone();
            let config = config.clone();

            move || {
                while let Some(child) = shares_list.first_child() {
                    shares_list.remove(&child);
                }

                let my_shares = crate::shares::get_shares(&config);

                if my_shares.is_empty() {
                    let placeholder_row = adw::ActionRow::builder()
                        .title(&*crate::i18n::tr("account.no_folders_shared"))
                        .subtitle(&*crate::i18n::tr("account.click_to_share"))
                        .build();
                    shares_list.append(&placeholder_row);
                } else {
                    for (share_name, share_path) in my_shares {
                        let row = adw::ActionRow::builder()
                            .title(&share_name)
                            .subtitle(&share_path)
                            .build();

                        let folder_img =
                            gtk::Image::from_resource("/com/icecommander/gtk/fileexplorer.svg");
                        folder_img.set_pixel_size(24);
                        row.add_prefix(&folder_img);

                        let edit_img =
                            gtk::Image::from_resource("/com/icecommander/gtk/edit-pencil.svg");
                        edit_img.set_pixel_size(24);
                        let edit_btn = Button::builder()
                            .child(&edit_img)
                            .css_classes(["flat"])
                            .tooltip_text(&*crate::i18n::tr("account.change_folder_path"))
                            .build();

                        let share_name_edit = share_name.clone();
                        let net_tx_clone = net_tx_dialog.clone();
                        let my_info_clone = my_info_dialog.clone();
                        let on_res_changed_clone = on_res_changed.clone();
                        let populate_shares_weak_clone = populate_shares_weak.clone();
                        let shares_list_root_clone = shares_list_root.clone();
                        let config_edit = config.clone();

                        edit_btn.connect_clicked(move |_| {
                            if let Some(win) = shares_list_root_clone
                                .root()
                                .and_then(|r| r.downcast::<gtk::Window>().ok())
                            {
                                let win_clone = win.clone();
                                let share_name_inner = share_name_edit.clone();
                                let net_tx_dialog_inner = net_tx_clone.clone();
                                let my_info_dialog_inner = my_info_clone.clone();
                                let on_res_changed_inner = on_res_changed_clone.clone();
                                let populate_shares_weak_inner = populate_shares_weak_clone.clone();
                                let config_inner = config_edit.clone();

                                crate::utils::select_folder(
                                    &win_clone,
                                    &*crate::i18n::tr("account.select_new_folder_path"),
                                    move |path| {
                                        let new_path = path.to_string_lossy().to_string();
                                        crate::shares::set_share_path(
                                            &config_inner,
                                            &share_name_inner,
                                            &new_path,
                                        );
                                        crate::shares::reload_resources(
                                            &config_inner,
                                            &my_info_dialog_inner.id,
                                            &net_tx_dialog_inner,
                                        );

                                        if let Some(cb) = &on_res_changed_inner {
                                            cb();
                                        }

                                        if let Some(rc) = populate_shares_weak_inner.upgrade() {
                                            if let Some(f) = rc.borrow().as_ref() {
                                                f();
                                            }
                                        }
                                    },
                                );
                            }
                        });
                        row.add_suffix(&edit_btn);

                        let del_btn_img =
                            gtk::Image::from_resource("/com/icecommander/gtk/close.svg");
                        del_btn_img.set_pixel_size(24);
                        let del_btn = Button::builder()
                            .child(&del_btn_img)
                            .css_classes(["flat", "destructive-action"])
                            .tooltip_text(&*crate::i18n::tr("account.stop_sharing"))
                            .build();

                        let share_name_del = share_name.clone();
                        let net_tx_del = net_tx_dialog.clone();
                        let my_info_del = my_info_dialog.clone();
                        let on_res_changed_del = on_res_changed.clone();
                        let populate_shares_weak_del = populate_shares_weak.clone();
                        let config_del = config.clone();

                        del_btn.connect_clicked(move |_| {
                            crate::shares::remove_share(&config_del, &share_name_del);
                            crate::shares::reload_resources(
                                &config_del,
                                &my_info_del.id,
                                &net_tx_del,
                            );

                            if let Some(cb) = &on_res_changed_del {
                                cb();
                            }

                            if let Some(rc) = populate_shares_weak_del.upgrade() {
                                if let Some(f) = rc.borrow().as_ref() {
                                    f();
                                }
                            }
                        });
                        row.add_suffix(&del_btn);

                        shares_list.append(&row);
                    }
                }
            }
        };

        *populate_shares_rc.borrow_mut() =
            Some(std::boxed::Box::new(populate_shares.clone()) as std::boxed::Box<dyn Fn()>);
        populate_shares();

        let populate_shares_weak_add = Rc::downgrade(&populate_shares_rc);
        let populate_shares_keepalive = populate_shares_rc.clone();
        let shares_list_root_add = shares_list.clone();
        let net_tx_add = net_tx_dialog.clone();
        let my_info_add = my_info_dialog.clone();
        let on_res_changed_add = on_resources_changed.clone();
        let config_add = config.clone();

        add_share_btn.connect_clicked(move |_| {
            let _keepalive = &populate_shares_keepalive;
            if let Some(win) = shares_list_root_add
                .root()
                .and_then(|r| r.downcast::<gtk::Window>().ok())
            {
                let win_clone = win.clone();
                let net_tx_dialog_inner = net_tx_add.clone();
                let my_info_dialog_inner = my_info_add.clone();
                let on_res_changed_inner = on_res_changed_add.clone();
                let populate_shares_weak_inner = populate_shares_weak_add.clone();
                let config_inner = config_add.clone();

                crate::utils::select_folder(&win, &*crate::i18n::tr("account.select_folder_to_share"), move |path| {
                    let path_str = path.to_string_lossy().to_string();
                    let default_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| crate::i18n::tr("account.shared_folder_default").to_string());

                    let entry_dialog = adw::MessageDialog::new(
                        Some(&win_clone),
                        Some(&*crate::i18n::tr("account.share_resource_name")),
                        None,
                    );
                    entry_dialog.set_body(&*crate::i18n::tr("account.enter_resource_name"));

                    let entry = gtk::Entry::builder()
                        .placeholder_text(&*crate::i18n::tr("account.resource_name_placeholder"))
                        .text(&default_name)
                        .build();

                    entry_dialog.set_extra_child(Some(&entry));
                    entry_dialog.add_response("cancel", &*crate::i18n::tr("account.cancel"));
                    entry_dialog.add_response("share", &*crate::i18n::tr("account.share"));
                    entry_dialog.set_default_response(Some("share"));
                    entry_dialog
                        .set_response_appearance("share", adw::ResponseAppearance::Suggested);

                    let path_str_dialog = path_str.clone();
                    let default_name_dialog = default_name.clone();
                    let net_tx_inner_cb = net_tx_dialog_inner.clone();
                    let my_info_inner_cb = my_info_dialog_inner.clone();
                    let on_res_changed_inner_cb = on_res_changed_inner.clone();
                    let populate_shares_weak_inner_cb = populate_shares_weak_inner.clone();
                    let config_cb = config_inner.clone();

                    entry_dialog.connect_response(None, move |d, response| {
                        if response == "share" {
                            let mut name = entry.text().to_string();
                            if name.is_empty() {
                                name = default_name_dialog.clone();
                            }

                            crate::shares::add_share(&config_cb, &name, &path_str_dialog);
                            crate::shares::reload_resources(
                                &config_cb,
                                &my_info_inner_cb.id,
                                &net_tx_inner_cb,
                            );

                            if let Some(cb) = &on_res_changed_inner_cb {
                                cb();
                            }

                            if let Some(rc) = populate_shares_weak_inner_cb.upgrade() {
                                if let Some(f) = rc.borrow().as_ref() {
                                    f();
                                }
                            }
                        }
                        d.close();
                    });
                    entry_dialog.present();
                });
            }
        });

        root_box.upcast::<gtk::Widget>()
    } else {
        #[cfg(not(feature = "nodeinnet"))]
        {
            let lbl = Label::new(Some("Authentication is available in the NodeInNet build only."));
            lbl.add_css_class("dim-label");
            return lbl.upcast::<gtk::Widget>();
        }
        #[cfg(feature = "nodeinnet")]
        {
        use node_auth::{LoginInit, LoginInput, LoginModel, LoginOutput};
        use relm4::prelude::*;

        let net_tx_inner = net_tx_dialog.clone();
        let my_info_inner = my_info_dialog.clone();
        let login_lbl_inner = login_lbl_dialog.clone();
        let close_cb = close_callback.clone();
        let active_dialog_graph_login = active_dialog_graph.clone();

        let graph_widget = crate::netgraph::NetGraph::new(
            gtk_graph_ui::NetworkGraphInit {
                icon_size: 40,
                interactive: false,
                show_names: true,
                show_local_node: true,
            },
            None,
        );
        graph_widget.root.set_size_request(350, 220);
        graph_widget.root.set_vexpand(true);
        graph_widget.root.set_hexpand(true);

        let (out_tx, out_rx) = relm4::channel::<LoginOutput>();
        let login_panel = LoginModel::builder()
            .launch(LoginInit {
                logo_resource: "/com/icecommander/gtk/nodeinnet-logo.svg".to_string(),
                prefix: String::new(),
                my_node_id: my_info_dialog.id.clone(),
                app_version: common::version::APP_VERSION.to_string(),
                config: config.clone(),
                graph_widget: Some(graph_widget.root.clone().upcast::<gtk::Widget>()),
                update_graph: None,
            })
            .forward(&out_tx, |o| o);
        let panel_widget = login_panel.widget().clone();
        let login_panel = Rc::new(login_panel);

        {
            let login_panel = login_panel.clone();
            graph_widget.set_on_peers(move |peers| {
                let _ = login_panel
                    .sender()
                    .send(LoginInput::PeersChanged(peers.to_vec()));
            });
        }

        graph_widget.update_peers(&online_nodes_dialog.borrow(), &my_info_dialog.id);
        *active_dialog_graph_login.borrow_mut() = Some(graph_widget.clone());

        {
            let config_login = config.clone();
            let login_panel_loop = login_panel.clone();
            glib::spawn_future_local(async move {
                while let Some(out) = out_rx.recv().await {
                    match out {
                        LoginOutput::SubmitLogin { login, password, guest, .. } => {
                            match client_core::auth::login(&api_base, &login, &password, config_login.turn_region()).await {
                                Ok(resp) => {
                                    config_login.set("app.refresh_token", resp.refresh_token.clone());
                                    config_login.set("app.account_login", login.clone());
                                    config_login.set("app.is_guest", guest);
                                    config_login.set("app.premium", resp.premium != 0);
                                    config_login.save();
                                    let _ = login_panel_loop.sender().send(LoginInput::LoginSucceeded);

                                    login_lbl_inner.set_text("Connecting...");
                                    match client_core::auth::refresh_access_token(
                                        &api_base,
                                        &resp.refresh_token,
                                        config_login.turn_region(),
                                    )
                                    .await
                                    {
                                        Ok(refresh) => {
                                            config_login.set("app.premium", refresh.premium != 0);
                                            config_login.save();
                                            let url = format!(
                                                "{}?token={}&session_id={}",
                                                refresh.ws_url, refresh.access_token, my_info_inner.id
                                            );
                                            let _ = net_tx_inner
                                                .send(client_core::NetCmd::Connect(
                                                    url,
                                                    my_info_inner.clone(),
                                                    refresh.turn,
                                                ))
                                                .await;
                                        }
                                        Err(_) => login_lbl_inner.set_text("Disconnected"),
                                    }
                                }
                                Err(e) => {
                                    let _ = login_panel_loop.sender().send(LoginInput::LoginFailed(e));
                                }
                            }
                        }
                        LoginOutput::TransitionFinished => close_cb(),
                    }
                }
            });
        }

        {
            let keepalive = login_panel.clone();
            panel_widget.connect_destroy(move |_| {
                let _ = &keepalive;
            });
        }

        panel_widget.upcast::<gtk::Widget>()
        }
    }
}

pub fn show_account_dialog(
    window: &gtk::Window,
    config: client_config::AppConfig,
    ws_state: &Rc<RefCell<client_core::WsState>>,
    active_dialog_graph: &Rc<RefCell<Option<crate::netgraph::NetGraph>>>,
    online_nodes: &Rc<RefCell<Vec<nodeinnet_p2p::NodeInfo>>>,
    my_info: &nodeinnet_p2p::NodeInfo,
    net_tx: &crate::core::NetCmdSender,
    login_btn_label: &Label,
    on_resources_changed: Option<Rc<dyn Fn()>>,
) {
    let ws_state_dialog = ws_state.clone();
    let is_connected = *ws_state_dialog.borrow() == client_core::WsState::Connected;
    let width = if is_connected { 450 } else { 400 };
    let height = if is_connected { 650 } else { 450 };

    let dialog = gtk::Window::builder()
        .title(&*crate::i18n::tr("account.title"))
        .transient_for(window)
        .modal(true)
        .default_width(width)
        .default_height(height)
        .resizable(false)
        .build();

    let dialog_to_close = dialog.clone();
    let close_cb = Rc::new(move || {
        dialog_to_close.close();
    });

    let widget = create_account_widget(
        config,
        ws_state,
        active_dialog_graph,
        online_nodes,
        my_info,
        net_tx,
        login_btn_label,
        close_cb,
        on_resources_changed,
    );

    dialog.set_child(Some(&widget));

    let active_dialog_graph_close = active_dialog_graph.clone();
    dialog.connect_close_request(move |_| {
        *active_dialog_graph_close.borrow_mut() = None;
        glib::Propagation::Proceed
    });

    dialog.present();
}
