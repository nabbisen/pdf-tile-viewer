// Expose compile-time TARGET triple as env var for BuildInfo::from_compile_time().
fn main() {
    println!(
        "cargo:rustc-env=TARGET={}",
        std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string())
    );
}
