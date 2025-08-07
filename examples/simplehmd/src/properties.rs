// Properties helper module for setting device properties
// Since IVRProperties uses batch operations, we'll simplify this for now

use openvr_driver_bindings::root::vr::PropertyContainerHandle_t;
use std::ffi::c_void;

// Global storage for VRProperties interface pointer
static mut VR_PROPERTIES: Option<*mut c_void> = None;

/// Set the VRProperties interface pointer (called during init)
pub unsafe fn set_vr_properties(properties: *mut c_void) {
    VR_PROPERTIES = Some(properties);
    eprintln!("SimpleHMD: VRProperties interface stored: {:?}", properties);
}

/// Get a property container for a tracked device
pub unsafe fn get_property_container(device_index: u32) -> PropertyContainerHandle_t {
    if let Some(properties) = VR_PROPERTIES {
        let properties_interface =
            properties as *mut openvr_driver_bindings::root::vr::IVRProperties;
        if !properties_interface.is_null() {
            let vtable = (*properties_interface).vtable_;
            if !vtable.is_null() {
                let get_container = (*vtable).IVRProperties_TrackedDeviceToPropertyContainer;
                return get_container(properties_interface, device_index);
            }
        }
    }
    0 // Invalid handle
}

/// Helper to set all required HMD properties
/// Note: Since IVRProperties uses batch operations (ReadPropertyBatch/WritePropertyBatch),
/// and we don't have the proper structs set up for that yet, we'll log what should be set.
/// In a production driver, you would use WritePropertyBatch to set all these at once.
pub unsafe fn set_hmd_properties(
    device_index: u32,
    model_number: &str,
    serial_number: &str,
    display_frequency: f32,
    ipd: f32,
) {
    let container = get_property_container(device_index);
    eprintln!(
        "SimpleHMD: Got property container {} for device {}",
        container, device_index
    );

    if container == 0 {
        eprintln!("SimpleHMD: WARNING: Invalid property container, properties cannot be set!");
        eprintln!("SimpleHMD: This may cause error 496 (display connection failed)");
    }

    // Log the properties that should be set via WritePropertyBatch
    eprintln!("SimpleHMD: Properties that need to be set (via WritePropertyBatch):");
    eprintln!("  String Properties:");
    eprintln!("    - Prop_ModelNumber_String (1003): {}", model_number);
    eprintln!("    - Prop_SerialNumber_String (1001): {}", serial_number);

    eprintln!("  Float Properties:");
    eprintln!(
        "    - Prop_DisplayFrequency_Float (2013): {} Hz",
        display_frequency
    );
    eprintln!("    - Prop_UserIpdMeters_Float (2003): {} meters", ipd);
    eprintln!("    - Prop_SecondsFromVsyncToPhotons_Float (2001): 0.11 seconds");
    eprintln!("    - Prop_DisplayGCScale_Float (2041): 1.0");
    eprintln!("    - Prop_DisplayGCOffset_Float (2038): 0.0");

    eprintln!("  Bool Properties (CRITICAL for error 496):");
    eprintln!(
        "    - Prop_IsOnDesktop_Bool (2070): true [CRITICAL - must be true for extended mode]"
    );
    eprintln!("    - Prop_DisplayDebugMode_Bool (2085): true [allows non-direct mode]");

    eprintln!("  Int32 Properties:");
    eprintln!("    - Prop_EdidVendorID_Int32 (2050): 0x0EAD");
    eprintln!("    - Prop_EdidProductID_Int32 (2051): 0x1234");
    eprintln!("    - Prop_DisplayMCType_Int32 (2086): 0");
    eprintln!("    - Prop_DisplayGCType_Int32 (2037): 1023");

    eprintln!("\nSimpleHMD: NOTE: Properties are not actually being set due to batch API");
    eprintln!("SimpleHMD: This is likely causing error 496 (failed to connect to headset display)");
    eprintln!(
        "SimpleHMD: To fix this, implement WritePropertyBatch with proper PropertyWrite_t structs"
    );
}
