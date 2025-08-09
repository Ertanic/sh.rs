use copy_to_output::copy_to_output;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=templates");
    copy_to_output("templates", &env::var("PROFILE").unwrap()).unwrap();
}