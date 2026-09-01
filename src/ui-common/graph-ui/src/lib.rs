mod i18n;

pub mod graph_view;
pub mod welcome_view;

pub use graph_view::{
    GraphNode, NetworkGraphInit, NetworkGraphInput, NetworkGraphModel, NetworkGraphOutput, PeerLink,
};
pub use welcome_view::{
    WelcomePanelInit, WelcomePanelInput, WelcomePanelModel, WelcomePanelOutput,
};

use std::sync::Once;

static RESOURCES_INIT: Once = Once::new();

pub fn init_resources() {
    RESOURCES_INIT.call_once(|| {
        gtk::gio::resources_register_include!("graph.gresource")
            .expect("Failed to register graph-ui resources");
    });
}

static STYLES_INIT: Once = Once::new();

pub fn init_styles() {
    STYLES_INIT.call_once(|| {
        let provider = gtk::CssProvider::new();
        provider.load_from_string(
            "
            .title-1 { font-size: 24pt; font-weight: bold; }
            .title-3 { font-size: 16pt; }
            .accent { background: linear-gradient(to right, #fbbf24, #d97706); color: white; padding: 4px 8px; border-radius: 6px; font-weight: 900; }
        ",
        );
        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });
}
