fn main() {
    println!("cargo:rerun-if-changed=locales");
    println!("cargo:rerun-if-changed=assets/node-auth.gresource.xml");
    glib_build_tools::compile_resources(
        &["assets"],
        "assets/node-auth.gresource.xml",
        "node-auth.gresource",
    );
}
