use std::env;

fn main() {
    let mut bridge_builder = cxx_build::bridge("src/ffi.rs");
    bridge_builder.cpp(true).flag_if_supported("-std=c++11");
    match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "macos" => {
            // The Rack C++ SDK uses -mmacosx-version-min=10.9 when building
            // and linking, let's try to match it in the cxx bridge and the
            // Rust staticlib.
            bridge_builder.flag("-mmacosx-version-min=10.9");
            println!("cargo::rustc-env=MACOSX_DEPLOYMENT_TARGET=10.9");
        }
        _ => {}
    }
    bridge_builder.compile("potential");
}
