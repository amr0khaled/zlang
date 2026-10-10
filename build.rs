fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_PROFILE");
    let profile = std::env::var("CARGO_PROFILE").unwrap_or_else(|_| "dev".to_string());
    if profile == "release" {
        println!("cargo:rustc-cfg=is_release");
    }
}
