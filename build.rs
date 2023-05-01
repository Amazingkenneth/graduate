#[cfg(target_os = "windows")]
fn main() {
    extern crate embed_resource;
    embed_resource::compile("./src/runtime/icon.rc", embed_resource::NONE);
}

#[cfg(not(target_os = "windows"))]
fn main() {}
