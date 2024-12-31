fn main() {
    println!("cargo:rerun-if-env-changed=SHUTTLE");

    if matches!(std::env::var("SHUTTLE").as_deref(), Ok("true")) {
        println!("cargo:rustc-env=SQLX_OFFLINE=true");
    };
}