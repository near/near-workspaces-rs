fn main() {
    println!("cargo:rerun-if-env-changed=NEAR_SANDBOX_BIN_PATH");

    let doc_build = cfg!(doc) || std::env::var("DOCS_RS").is_ok();
    let env_bin = std::env::var("NEAR_SANDBOX_BIN_PATH").is_ok();
    if !doc_build && !env_bin && cfg!(feature = "install") {
        // TODO Update commit to stable version once binaries are published correctly
        // Commit: https://github.com/near/nearcore/commit/eb2bbe1c3f51912c04462ce988aa496fab03d60e
        near_sandbox_utils::install_with_version("master/eb2bbe1c3f51912c04462ce988aa496fab03d60e")
            .expect("Could not install sandbox");
    }
}
