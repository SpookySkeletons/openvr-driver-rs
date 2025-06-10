use std::path::PathBuf;

fn main() {
    // Existing autocxx code
    let include_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("openvr/headers");

    let mut builder = autocxx_build::Builder::new("src/lib.rs", vec![include_path.clone()]);

    builder
        .build()
        .expect("Failed to build autocxx bindings")
        .flag_if_supported("-std=c++14")
        .compile("openvr-driver-bindings");

    // Compile C++ bridge with warnings suppressed
    println!("cargo:warning=Compiling C++ bridge...");

    cc::Build::new()
        .cpp(true)
        .file("c_bridge/rust_provider_wrapper.cpp")
        .include(&include_path)
        .include("c_bridge")
        .flag_if_supported("-std=c++14")
        .flag_if_supported("-fPIC")
        // Suppress C++ warnings
        .flag_if_supported("-w") // Suppress all warnings
        .flag_if_supported("-Wno-mismatched-new-delete")
        .flag_if_supported("-Wno-unused-parameter") // Specifically unused parameters
        .define("AUTOCXX_SUPPRESS_WARNINGS", None)
        .compile("rust-openvr-bridge");

    println!("cargo:warning=C++ bridge compiled successfully!");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=c_bridge/rust_provider_bridge.h");
    println!("cargo:rerun-if-changed=c_bridge/rust_provider_wrapper.cpp");
}
