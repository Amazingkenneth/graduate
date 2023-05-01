#[cfg(target_os = "windows")]
extern crate embed_resource;

fn main() {
    if cfg!(target_os = "windows") {
        embed_resource::compile("./src/runtime/icon.rc", embed_resource::NONE);
    }
}
