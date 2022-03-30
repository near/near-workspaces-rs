fn main() {
    if !cfg!(doc) && cfg!(feature = "install") {
        near_sandbox_utils::install().expect("Could not install sandbox");
    }
}
