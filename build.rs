fn main() {
    // Add ICO resources
    embed_resource::compile("assets/resources.rc", embed_resource::NONE)
        .manifest_optional()
        .unwrap();
    println!("cargo:rustc-link-arg-bin=tray-weather=/RES:assets/resources.res");
}
