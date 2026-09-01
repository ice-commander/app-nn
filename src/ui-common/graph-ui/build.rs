fn main() {
    println!("cargo:rerun-if-changed=locales");
    println!("cargo:rerun-if-changed=assets/graph.gresource.xml");
    glib_build_tools::compile_resources(
        &["assets"],
        "assets/graph.gresource.xml",
        "graph.gresource",
    );
}
