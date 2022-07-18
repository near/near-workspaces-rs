fn main() {
    let doc_build = cfg!(doc) || std::env::var("DOCS_RS").is_ok();
    let env_bin = std::env::var("NEAR_SANDBOX_BIN_PATH").is_ok();
    if !doc_build && !env_bin && cfg!(feature = "install") {
        // TODO Update commit to stable version once binaries are published correctly
        near_sandbox_utils::install().expect("Could not install sandbox");
    }
}
