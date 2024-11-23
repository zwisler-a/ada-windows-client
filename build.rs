extern crate embed_resource;
fn main() {
    embed_resource::compile("manifest.rc", embed_resource::NONE).manifest_optional().unwrap();
}