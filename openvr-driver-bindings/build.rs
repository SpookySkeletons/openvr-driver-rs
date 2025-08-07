use bindgen::callbacks::ParseCallbacks;
use std::path::Path;
use syn::parse_quote;

#[allow(unused_macros)]
macro_rules! dbg {
    ($($tokens:tt)*) => {
        for line in format!($($tokens)*).lines() {
            println!("cargo:warning={line}")
        }
    }
}

#[derive(Debug)]
struct Callbacks;

impl ParseCallbacks for Callbacks {
    fn add_derives(&self, _info: &bindgen::callbacks::DeriveInfo<'_>) -> Vec<String> {
        // Return empty vec since bindgen already adds Debug, Clone, etc. by default
        // We only add derives when needed for specific types
        Vec::new()
    }

    // Strip enum prefixes for cleaner Rust names
    fn enum_variant_name(
        &self,
        enum_name: Option<&str>,
        original_variant_name: &str,
        _variant_value: bindgen::callbacks::EnumVariantValue,
    ) -> Option<String> {
        let enum_name = enum_name?;

        // Handle k_ prefix pattern
        if let Some(name) = original_variant_name.strip_prefix("k_") {
            return Some(name.to_string());
        }

        // Handle VR prefix pattern
        if let Some(name) = original_variant_name.strip_prefix("VR") {
            if name.starts_with(enum_name.strip_prefix("EVR").unwrap_or(enum_name)) {
                return name
                    .strip_prefix(enum_name.strip_prefix("EVR").unwrap_or(enum_name))
                    .and_then(|s| s.strip_prefix('_'))
                    .map(|s| s.to_string());
            }
        }

        // Handle enum name prefix
        if let Some(name) = original_variant_name.strip_prefix(enum_name) {
            return name
                .strip_prefix('_')
                .map(|s| s.to_string())
                .or_else(|| Some(name.to_string()));
        }

        None
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let header_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("headers");
    let header_path = header_dir.join("openvr_driver.h");

    println!("cargo:rerun-if-changed={}", header_path.display());

    // Driver interfaces we want to generate bindings for
    const INTERFACES: &[&str] = &[
        "ITrackedDeviceServerDriver",
        "IServerTrackedDeviceProvider",
        "IVRDriverContext",
        "IVRServerDriverHost",
        "IVRWatchdogProvider",
        "IVRCompositorDriverHost",
        "IVRDriverManager",
        "IVRResources",
        "IVRDriverInput",
    ];

    let mut builder = bindgen::builder()
        .header(header_path.to_str().unwrap())
        .clang_args([
            "-x",
            "c++",
            "-fparse-all-comments",
            "-DOPENVR_DRIVER_HEADER",
        ])
        .enable_cxx_namespaces()
        .derive_default(true)
        .vtable_generation(true)
        .generate_cstr(true)
        .generate_comments(true)
        .layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(Callbacks));

    // Allowlist the interfaces we care about
    for interface in INTERFACES {
        builder = builder
            .allowlist_type(format!("vr::{interface}"))
            .allowlist_item(format!("vr::{interface}_Version"));
    }

    // Include essential types and constants
    builder = builder
        .allowlist_item("vr::k_.*") // Constants
        .allowlist_item("vr::VR.*") // VR functions
        .allowlist_type("vr::EVR.*") // Enums
        .allowlist_type("vr::.*Handle_t") // Handle types
        .allowlist_type("vr::TrackedDevice.*") // Device types
        .allowlist_type("vr::HmdMatrix.*") // Matrix types
        .allowlist_type("vr::HmdVector.*") // Vector types
        .allowlist_type("vr::HmdQuaternion.*") // Quaternion types
        .allowlist_type("vr::DriverPose_t") // Driver pose
        .allowlist_type("vr::VRControllerState_t") // Controller state
        .allowlist_type("vr::Prop_.*") // Properties
        .blocklist_function("vr::VR_.*") // We'll implement these ourselves
        .rustified_enum(".*");

    let bindings = builder.generate()?;

    // Process the bindings to add our vtable handling
    let syntax = syn::parse_file(&bindings.to_string())?;
    let processed = process_driver_types(syntax);

    let path = Path::new(&std::env::var("OUT_DIR")?).join("bindings.rs");
    std::fs::write(&path, processed)?;

    Ok(())
}

fn process_driver_types(mut syntax: syn::File) -> String {
    // Add our custom traits and implementations
    let additional_items: Vec<syn::Item> = vec![
        parse_quote! {
            pub trait InterfaceImpl {
                fn get_vtable(&self, version: &std::ffi::CStr) -> Option<*mut std::ffi::c_void>;
            }
        },
        parse_quote! {
            pub trait Inherits<T> {
                fn new_wrapped(this: &std::sync::Arc<Self>) -> VtableWrapper<T, Self>
                where
                    Self: Sized;
            }
        },
        parse_quote! {
            #[repr(C)]
            pub struct VtableWrapper<T, U> {
                pub base: T,
                pub this: std::sync::Arc<U>,
            }
        },
    ];

    syntax.items.extend(additional_items);

    // Convert back to string
    prettyplease::unparse(&syntax)
}
