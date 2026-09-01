use gtk_graph_ui::{NetworkGraphInit, NetworkGraphInput, NetworkGraphModel, NetworkGraphOutput};
use relm4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct NetGraph {
    pub root: gtk::Overlay,
    sender: relm4::Sender<NetworkGraphInput>,
    on_peers: Rc<RefCell<Option<Box<dyn Fn(&[nodeinnet_p2p::NodeInfo])>>>>,
    _controller: Rc<Controller<NetworkGraphModel>>,
}

impl NetGraph {
    pub fn new(init: NetworkGraphInit, on_click: Option<Box<dyn Fn(String)>>) -> Self {
        let (out_tx, out_rx) = relm4::channel::<NetworkGraphOutput>();
        let controller = NetworkGraphModel::builder()
            .launch(init)
            .forward(&out_tx, |o| o);
        let root = controller.widget().clone();
        let sender = controller.sender().clone();

        gtk::glib::spawn_future_local(async move {
            while let Some(out) = out_rx.recv().await {
                let NetworkGraphOutput::NodeClicked(id) = out;
                if let Some(cb) = on_click.as_ref() {
                    cb(id);
                }
            }
        });

        Self {
            root,
            sender,
            on_peers: Rc::new(RefCell::new(None)),
            _controller: Rc::new(controller),
        }
    }

    pub fn update_peers(&self, peers: &[nodeinnet_p2p::NodeInfo], my_id: &str) {
        let _ = self.sender.send(NetworkGraphInput::UpdatePeers {
            peers: peers.to_vec(),
            my_id: my_id.to_string(),
        });
        if let Some(cb) = self.on_peers.borrow().as_ref() {
            cb(peers);
        }
    }

    pub fn set_on_peers(&self, cb: impl Fn(&[nodeinnet_p2p::NodeInfo]) + 'static) {
        *self.on_peers.borrow_mut() = Some(Box::new(cb));
    }
}
