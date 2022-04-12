fn main() {
    let doc_build = cfg!(doc) || std::env::var("DOCS_RS").is_ok();
    if !doc_build && cfg!(feature = "install") {
        near_sandbox_utils::install().expect("Could not install sandbox");
    }
}
