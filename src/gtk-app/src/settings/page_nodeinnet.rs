use gtk::prelude::*;
use gtk::{Align, Box, Label};
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(not(feature = "nodeinnet"))]
use crate::core::client_core;
#[cfg(feature = "nodeinnet")]
use client_core;

pub(super) fn build(
    page_box: &Box,
    parent: &gtk::Window,
    config: client_config::AppConfig,
    ws_state: &Rc<RefCell<client_core::WsState>>,
    active_dialog_graph: &Rc<RefCell<Option<crate::netgraph::NetGraph>>>,
    online_nodes: &Rc<RefCell<Vec<nodeinnet_p2p::NodeInfo>>>,
    my_info: &nodeinnet_p2p::NodeInfo,
    net_tx: &crate::core::NetCmdSender,
    login_btn_label: &gtk::Label,
    on_connections_changed: Rc<dyn Fn() + 'static>,
) {
    let page_title = Label::builder()
        .label("<span size='x-large' weight='bold'>Node.In.Net Account</span>")
        .use_markup(true)
        .halign(Align::Start)
        .margin_bottom(16)
        .build();
    page_box.append(&page_title);

    let parent_clone = parent.clone();
    let close_cb = Rc::new(move || {
        parent_clone.close();
    });
    let widget = crate::account::create_account_widget(
        config,
        ws_state,
        active_dialog_graph,
        online_nodes,
        my_info,
        net_tx,
        login_btn_label,
        close_cb,
        Some(on_connections_changed),
    );
    page_box.append(&widget);
}
