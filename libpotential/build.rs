fn main() {
    cxx_build::bridge("src/ffi.rs")
        .cpp(true)
        // FIXME: get this from an env variable
        .flag("-mmacosx-version-min=10.9")
        .flag_if_supported("-std=c++11")
        .compile("potential");
}
