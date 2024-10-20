#[cfg(all(windows, feature = "ui"))]
extern crate embed_resource;

fn main() {
    #[cfg(all(windows, feature = "ui"))]
    embed_resource::compile("rc.rc", embed_resource::NONE);
}