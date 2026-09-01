use adw::prelude::*;
use nodeinnet_p2p::NodeInfo;
use gtk::glib;
use relm4::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct GraphNode {
    pub id: String,
    pub name: String,
    pub os: String,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub is_local: bool,
    pub is_online: bool,
    pub link: Option<PeerLink>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerLink {
    Idle,
    Connecting,
    Connected,
    Failed,
}

pub struct NetworkGraphInit {
    pub icon_size: u32,
    pub interactive: bool,
    pub show_names: bool,
    pub show_local_node: bool,
}

impl Default for NetworkGraphInit {
    fn default() -> Self {
        Self {
            icon_size: 30,
            interactive: false,
            show_names: false,
            show_local_node: true,
        }
    }
}

#[derive(Debug)]
pub enum NetworkGraphInput {
    UpdatePeers { peers: Vec<NodeInfo>, my_id: String },
    UpdatePeerLinks(HashMap<String, PeerLink>),
    NodePressed(String),
    Mapped(bool),
}

#[derive(Debug)]
pub enum NetworkGraphOutput {
    NodeClicked(String),
}

pub struct NetworkGraphModel {
    nodes: Rc<RefCell<HashMap<String, GraphNode>>>,
    local_id: Rc<RefCell<String>>,
    icon_size: u32,
    interactive: bool,
    show_names: bool,
    show_local_node: bool,
    peers_generation: u64,
    sim_gate: Rc<Cell<bool>>,
    timer_armed: Rc<Cell<bool>>,
    images: Rc<RefCell<HashMap<String, gtk::Image>>>,
    labels: Rc<RefCell<HashMap<String, gtk::Label>>>,
    drawing_area: gtk::DrawingArea,
    fixed: gtk::Fixed,
}

impl NetworkGraphModel {
    fn arm_sim_timer_if_needed(&self) {
        if self.timer_armed.get() {
            return;
        }
        self.timer_armed.set(true);
        let gate = self.sim_gate.clone();
        let armed = self.timer_armed.clone();
        let nodes = self.nodes.clone();
        let local_id = self.local_id.clone();
        let images = self.images.clone();
        let labels = self.labels.clone();
        let drawing_area = self.drawing_area.clone();
        let fixed = self.fixed.clone();
        let icon_size = self.icon_size;
        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            if !gate.get() {
                armed.set(false);
                return glib::ControlFlow::Break;
            }
            simulation_step(&nodes, &local_id.borrow());
            move_icons(&nodes, &images, &labels, &fixed, &drawing_area, icon_size);
            drawing_area.queue_draw();
            glib::ControlFlow::Continue
        });
    }
}

pub struct NetworkGraphWidgets {
    drawing_area: gtk::DrawingArea,
    fixed: gtk::Fixed,
    rendered_generation: u64,
}

impl SimpleComponent for NetworkGraphModel {
    type Init = NetworkGraphInit;
    type Input = NetworkGraphInput;
    type Output = NetworkGraphOutput;
    type Root = gtk::Overlay;
    type Widgets = NetworkGraphWidgets;

    fn init_root() -> Self::Root {
        gtk::Overlay::new()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        crate::init_resources();
        root.set_hexpand(true);
        root.set_vexpand(true);

        let drawing_area = gtk::DrawingArea::new();
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);
        drawing_area.set_halign(gtk::Align::Fill);
        drawing_area.set_valign(gtk::Align::Fill);
        root.set_child(Some(&drawing_area));

        let fixed = gtk::Fixed::new();
        root.add_overlay(&fixed);

        let nodes = Rc::new(RefCell::new(HashMap::<String, GraphNode>::new()));
        let local_id = Rc::new(RefCell::new(String::new()));
        let images = Rc::new(RefCell::new(HashMap::<String, gtk::Image>::new()));
        let labels = Rc::new(RefCell::new(HashMap::<String, gtk::Label>::new()));

        let nodes_draw = nodes.clone();
        let local_id_draw = local_id.clone();
        drawing_area.set_draw_func(move |_area, cr, width, height| {
            if width <= 1 || height <= 1 {
                return;
            }

            let cx = width as f64 * 0.5;
            let cy = height as f64 * 0.5;
            let scale_factor = 0.5;

            let state = nodes_draw.borrow();
            let lid = local_id_draw.borrow();

            cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            let _ = cr.paint();

            if let Some(local_node) = state.get(&*lid) {
                for (id, node) in state.iter() {
                    if id != &*lid {
                        let p1_x = cx + (node.x - 500.0) * scale_factor;
                        let p1_y = cy + (node.y - 500.0) * scale_factor;
                        let p2_x = cx + (local_node.x - 500.0) * scale_factor;
                        let p2_y = cy + (local_node.y - 500.0) * scale_factor;

                        match (node.is_online, node.link) {
                            (false, _) => {
                                cr.set_source_rgba(0.6, 0.6, 0.6, 0.5); // gray
                                cr.set_dash(&[5.0, 5.0], 0.0);
                            }
                            (true, Some(PeerLink::Connected)) => {
                                cr.set_source_rgba(0.12, 0.61, 0.40, 0.75); // green
                                cr.set_dash(&[], 0.0);
                            }
                            (true, Some(PeerLink::Connecting)) => {
                                cr.set_source_rgba(0.79, 0.54, 0.0, 0.7); // amber
                                cr.set_dash(&[4.0, 3.0], 0.0);
                            }
                            (true, Some(PeerLink::Failed)) => {
                                cr.set_source_rgba(0.82, 0.14, 0.17, 0.7); // red
                                cr.set_dash(&[], 0.0);
                            }
                            (true, Some(PeerLink::Idle) | None) => {
                                cr.set_source_rgba(0.23, 0.51, 0.96, 0.5); // blue
                                cr.set_dash(&[], 0.0);
                            }
                        }
                        cr.set_line_width(3.0 * scale_factor);
                        cr.move_to(p1_x, p1_y);
                        cr.line_to(p2_x, p2_y);
                        let _ = cr.stroke();
                        cr.set_dash(&[], 0.0); // reset dash so online links stay solid
                    }
                }
            }
        });

        let sender_map = sender.clone();
        root.connect_map(move |_| {
            sender_map.input(NetworkGraphInput::Mapped(true));
        });
        let sender_unmap = sender.clone();
        root.connect_unmap(move |_| {
            sender_unmap.input(NetworkGraphInput::Mapped(false));
        });

        let model = NetworkGraphModel {
            nodes,
            local_id,
            icon_size: init.icon_size,
            interactive: init.interactive,
            show_names: init.show_names,
            show_local_node: init.show_local_node,
            peers_generation: 0,
            sim_gate: Rc::new(Cell::new(false)),
            timer_armed: Rc::new(Cell::new(false)),
            images,
            labels,
            drawing_area: drawing_area.clone(),
            fixed: fixed.clone(),
        };
        let widgets = NetworkGraphWidgets {
            drawing_area,
            fixed,
            rendered_generation: 0,
        };
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            NetworkGraphInput::UpdatePeers { peers, my_id } => {
                {
                    let mut state = self.nodes.borrow_mut();
                    *self.local_id.borrow_mut() = my_id.clone();

                    let incoming_ids: HashSet<_> = peers.iter().map(|n| n.id.clone()).collect();
                    state.retain(|id, _| incoming_ids.contains(id) || *id == my_id);

                    for (i, peer) in peers.iter().enumerate() {
                        if peer.id == my_id {
                            continue;
                        }
                        if !state.contains_key(&peer.id) {
                            let angle = (std::f64::consts::PI * 2.0 * (i as f64))
                                / (peers.len().max(1) as f64);
                            let distance = 300.0;
                            let x = 500.0 + angle.cos() * distance;
                            let y = 500.0 + angle.sin() * distance;

                            state.insert(
                                peer.id.clone(),
                                GraphNode {
                                    id: peer.id.clone(),
                                    name: peer.name.clone(),
                                    os: peer.os.clone(),
                                    x,
                                    y,
                                    vx: 0.0,
                                    vy: 0.0,
                                    is_local: false,
                                    is_online: peer.is_online,
                                    link: None,
                                },
                            );
                        } else if let Some(node) = state.get_mut(&peer.id) {
                            node.is_online = peer.is_online;
                        }
                    }

                    if self.show_local_node {
                        let os_str = std::env::consts::OS.to_string();
                        let display_name = match os_str.as_str() {
                            "windows" => crate::i18n::tr("graph.this_windows"),
                            "macos" => crate::i18n::tr("graph.this_macos"),
                            "linux" => crate::i18n::tr("graph.this_linux"),
                            "android" => crate::i18n::tr("graph.this_android"),
                            "ios" => crate::i18n::tr("graph.this_ios"),
                            _ => crate::i18n::tr("graph.this_device"),
                        }
                        .to_string();

                        if let Some(node) = state.get_mut(&my_id) {
                            node.name = display_name;
                            node.os = os_str;
                            node.is_local = true;
                            node.is_online = true;
                        } else {
                            state.insert(
                                my_id.clone(),
                                GraphNode {
                                    id: my_id.clone(),
                                    name: display_name,
                                    os: os_str,
                                    x: 500.0,
                                    y: 500.0,
                                    vx: 0.0,
                                    vy: 0.0,
                                    is_local: true,
                                    is_online: true,
                                    link: None,
                                },
                            );
                        }
                    } else {
                        state.remove(&my_id);
                    }
                }
                self.peers_generation += 1;
            }
            NetworkGraphInput::UpdatePeerLinks(links) => {
                if let Ok(mut state) = self.nodes.try_borrow_mut() {
                    for (id, link) in links {
                        if let Some(node) = state.get_mut(&id) {
                            node.link = Some(link);
                        }
                    }
                }
                self.drawing_area.queue_draw();
            }
            NetworkGraphInput::NodePressed(id) => {
                if !self.interactive || id == *self.local_id.borrow() {
                    return;
                }
                let is_online = self
                    .nodes
                    .borrow()
                    .get(&id)
                    .map(|n| n.is_online)
                    .unwrap_or(false);
                if is_online {
                    let _ = sender.output(NetworkGraphOutput::NodeClicked(id));
                }
            }
            NetworkGraphInput::Mapped(mapped) => {
                self.sim_gate.set(mapped);
                if mapped {
                    self.arm_sim_timer_if_needed();
                }
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: ComponentSender<Self>) {
        if widgets.rendered_generation == self.peers_generation {
            return;
        }
        widgets.rendered_generation = self.peers_generation;

        let state = self.nodes.borrow();
        let lid = self.local_id.borrow();
        let mut images = self.images.borrow_mut();
        let mut labels = self.labels.borrow_mut();

        let width = widgets.drawing_area.width() as f64;
        let height = widgets.drawing_area.height() as f64;
        let cx = width * 0.5;
        let cy = height * 0.5;
        let scale_factor = 0.5;

        for (id, node) in state.iter() {
            let px = cx + (node.x - 500.0) * scale_factor;
            let py = cy + (node.y - 500.0) * scale_factor;
            let offset = (self.icon_size as f64) / 2.0;

            if !images.contains_key(id) {
                let os_key = if id == &*lid {
                    os_icon_key(std::env::consts::OS)
                } else {
                    os_icon_key(&node.os.to_lowercase())
                };
                let resource_path = format!("/com/graph-ui/gtk/{}.svg", os_key);
                let image = gtk::Image::from_resource(&resource_path);
                image.set_pixel_size(self.icon_size as i32);

                if self.interactive {
                    let click_controller = gtk::GestureClick::new();
                    let node_id = id.clone();
                    let sender_click = sender.clone();
                    click_controller.connect_pressed(move |_, _, _, _| {
                        sender_click.input(NetworkGraphInput::NodePressed(node_id.clone()));
                    });
                    image.add_controller(click_controller);
                }

                widgets.fixed.put(&image, px - offset, py - offset);
                images.insert(id.clone(), image);

                if self.show_names {
                    let label = gtk::Label::builder()
                        .label(&node.name)
                        .halign(gtk::Align::Center)
                        .width_request(120)
                        .lines(1)
                        .ellipsize(gtk::pango::EllipsizeMode::End)
                        .build();
                    label.add_css_class("dim-label");

                    widgets.fixed.put(&label, px - 60.0, py + offset + 4.0);
                    labels.insert(id.clone(), label);
                }
            }

            let Some(image) = images.get(id) else {
                continue;
            };
            if node.is_online {
                image.remove_css_class("offline-grayscale");
                if id == &*lid {
                    image.set_opacity(1.0);
                } else {
                    image.set_opacity(0.65);
                }
                if self.interactive {
                    image.set_cursor_from_name(Some("pointer"));
                }
            } else {
                image.add_css_class("offline-grayscale");
                image.set_opacity(0.25);
                image.set_cursor_from_name(None);
            }
        }

        let to_remove: Vec<String> = images
            .keys()
            .filter(|id| !state.contains_key(*id))
            .cloned()
            .collect();
        for id in to_remove {
            if let Some(image) = images.remove(&id) {
                widgets.fixed.remove(&image);
            }
            if let Some(label) = labels.remove(&id) {
                widgets.fixed.remove(&label);
            }
        }

        widgets.drawing_area.queue_draw();
    }
}

fn os_icon_key(os: &str) -> &'static str {
    match os {
        "windows" => "os-win",
        "macos" => "os-mac",
        "linux" => "os-linux",
        "android" => "os-android",
        "ios" => "os-iphone",
        _ => "os-console",
    }
}

fn simulation_step(nodes: &Rc<RefCell<HashMap<String, GraphNode>>>, lid: &str) {
    let mut forces = HashMap::new();
    {
        let state = nodes.borrow();
        for id in state.keys() {
            forces.insert(id.clone(), (0.0, 0.0));
        }

        for (id_a, node_a) in state.iter() {
            for (id_b, node_b) in state.iter() {
                if id_a != id_b {
                    let dx = node_a.x - node_b.x;
                    let dy = node_a.y - node_b.y;
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq > 0.1 {
                        let force = 5000.0 / dist_sq;
                        let dist = dist_sq.sqrt();
                        if let Some(f) = forces.get_mut(id_a) {
                            f.0 += (dx / dist) * force;
                            f.1 += (dy / dist) * force;
                        }
                    }
                }
            }
        }

        if let Some(node) = state.get(lid) {
            let dx = 500.0 - node.x;
            let dy = 500.0 - node.y;
            if let Some(f) = forces.get_mut(lid) {
                f.0 += dx * 0.01;
                f.1 += dy * 0.01;
            }
        }

        if let Some(l_node) = state.get(lid) {
            let lx = l_node.x;
            let ly = l_node.y;
            for (id, node) in state.iter() {
                if id != lid {
                    let dx = lx - node.x;
                    let dy = ly - node.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist > 0.1 {
                        let force = (dist - 200.0) * 0.02;
                        let fx = (dx / dist) * force;
                        let fy = (dy / dist) * force;

                        if let Some(f_node) = forces.get_mut(id) {
                            f_node.0 += fx;
                            f_node.1 += fy;
                        }

                        if let Some(f_local) = forces.get_mut(lid) {
                            f_local.0 -= fx;
                            f_local.1 -= fy;
                        }
                    }
                }
            }
        }
    }

    let mut state = nodes.borrow_mut();
    for (id, node) in state.iter_mut() {
        if let Some(f) = forces.get(id) {
            node.vx = (node.vx + f.0) * 0.8;
            node.vy = (node.vy + f.1) * 0.8;
            node.x += node.vx;
            node.y += node.vy;
        }
    }
    for node in state.values_mut() {
        let dx = node.x - 500.0;
        let dy = node.y - 500.0;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist > 350.0 {
            let scale = 350.0 / dist;
            node.x = 500.0 + dx * scale;
            node.y = 500.0 + dy * scale;
        }
    }
}

fn move_icons(
    nodes: &Rc<RefCell<HashMap<String, GraphNode>>>,
    images: &Rc<RefCell<HashMap<String, gtk::Image>>>,
    labels: &Rc<RefCell<HashMap<String, gtk::Label>>>,
    fixed: &gtk::Fixed,
    drawing_area: &gtk::DrawingArea,
    icon_size: u32,
) {
    let width = drawing_area.width() as f64;
    let height = drawing_area.height() as f64;
    if width <= 1.0 || height <= 1.0 {
        return;
    }
    let cx = width * 0.5;
    let cy = height * 0.5;
    let scale_factor = 0.5;
    let offset = (icon_size as f64) / 2.0;

    let state = nodes.borrow();
    let images = images.borrow();
    let labels = labels.borrow();
    for (id, node) in state.iter() {
        let px = cx + (node.x - 500.0) * scale_factor;
        let py = cy + (node.y - 500.0) * scale_factor;
        if let Some(image) = images.get(id) {
            fixed.move_(image, px - offset, py - offset);
        }
        if let Some(label) = labels.get(id) {
            fixed.move_(label, px - 60.0, py + offset + 4.0);
        }
    }
}
