fn main() {
    let doc_build = cfg!(doc) || std::env::var("DOCS_RS").is_ok();
    if !doc_build && cfg!(feature = "install") {
        // TODO Update commit to stable version once binaries are published correctly
        near_sandbox_utils::install_with_version("master/97c0410de519ecaca369aaee26f0ca5eb9e7de06")
            .expect("Could not install sandbox");
    }
}
