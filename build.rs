use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Automatically generate bindings for "HalonMTA.h"
    let bindings = bindgen::Builder::default()
        .header("/opt/halon/include/HalonMTA.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    tonic_prost_build::compile_protos("proto/rfc822.proto")?;

    Ok(())
}
