mod i18n;

pub mod login_view;

pub use login_view::{LoginInit, LoginInput, LoginModel, LoginOutput, UpdateGraphFn};

use std::sync::Once;

static RESOURCES_INIT: Once = Once::new();

pub fn init_resources() {
    RESOURCES_INIT.call_once(|| {
        gtk::gio::resources_register_include!("node-auth.gresource")
            .expect("Failed to register node-auth resources");
    });
}
