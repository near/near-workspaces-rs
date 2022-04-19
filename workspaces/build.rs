fn main() {
    let doc_build = cfg!(doc) || std::env::var("DOCS_RS").is_ok();
    if !doc_build && cfg!(feature = "install") {
        // using unwrap because all the useful error messages are hidden inside
        near_sandbox_utils::ensure_sandbox_bin().unwrap();
        // previously the next line was causing near sandbox to be installed every time cargo build/check was called
        // near_sandbox_utils::install().expect("Could not install sandbox");
    }
}
