fn main() {
    let doc_build = cfg!(doc) || std::env::var("DOCS_RS").is_ok();
    if !doc_build && cfg!(feature = "install") {
        match near_sandbox_utils::ensure_sandbox_bin(){
            Ok(p) => println!("Successfully installed sandbox in: {:?}", p),
            Err(e) => panic!("Could not install sandbox\nReason: {:?}", e),
        }
    }
}
