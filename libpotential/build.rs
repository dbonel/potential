use std::env;

fn main() {
    let mut bridge_builder = cxx_build::bridge("src/ffi.rs");
    bridge_builder.cpp(true).flag_if_supported("-std=c++11");
    match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "macos" => {
            bridge_builder.flag("-mmacosx-version-min=10.9");
        }
        _ => {}
    }
    bridge_builder.compile("potential");
}
