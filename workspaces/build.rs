fn main() {
    let doc_build = cfg!(doc) || std::env::var("DOCS_RS").is_ok();
    if !doc_build && cfg!(feature = "install") {
        near_sandbox_utils::install_with_version("1.26.0/57b6ac92b0e77deade8a6eef1a322272511aa5de")
            .expect("Could not install sandbox");
    }
}
