use copy_to_output::copy_to_output;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=templates");
    println!("cargo:rerun-if-changed=assets");
    copy_to_output("templates", &env::var("PROFILE").unwrap()).unwrap();
    copy_to_output("assets", &env::var("PROFILE").unwrap()).unwrap();
}