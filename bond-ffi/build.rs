fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let building_python_extension = std::env::var_os("PYO3_BUILD_EXTENSION_MODULE").is_some();

    if target_os == "macos" && building_python_extension {
        // Allow unresolved symbols (e.g., Python C-API) when producing `cdylib` on macOS for wheels.
        // This matches the typical "dynamic_lookup" strategy used by Python extension modules.
        println!("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup");
    }
}
