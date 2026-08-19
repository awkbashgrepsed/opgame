use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is set by cargo");
    let mut file = File::create(Path::new(&out_dir).join("gl_bindings.rs"))
        .expect("Failed to create the OpenGL bindings file");

    Registry::new(Api::Gl, (2, 1), Profile::Compatibility, Fallbacks::All, [])
        .write_bindings(GlobalGenerator, &mut file)
        .expect("Failed to generate the OpenGL bindings");
}
