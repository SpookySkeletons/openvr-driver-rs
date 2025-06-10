// Suppress warnings from autocxx-generated code
#![allow(warnings)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

use autocxx::prelude::*;

include_cpp! {
    #include "openvr_driver.h"

    // Basic math/geometry types
    generate!("vr::HmdMatrix34_t")
    generate!("vr::HmdMatrix33_t")
    generate!("vr::HmdMatrix44_t")
    generate!("vr::HmdVector3_t")
    generate!("vr::HmdVector4_t")
    generate!("vr::HmdVector3d_t")
    generate!("vr::HmdVector2_t")
    generate!("vr::HmdQuaternion_t")
    generate!("vr::HmdQuaternionf_t")
    generate!("vr::HmdColor_t")
    generate!("vr::HmdQuad_t")
    generate!("vr::HmdRect2_t")

    // Core VR types
    generate!("vr::VRBoneTransform_t")
    generate!("vr::DistortionCoordinates_t")
    generate!("vr::Texture_t")
    generate!("vr::VRTextureBounds_t")
    generate!("vr::VRTextureWithPose_t")
    generate!("vr::VRTextureDepthInfo_t")
    generate!("vr::VRTextureWithDepth_t")
    generate!("vr::VRTextureWithPoseAndDepth_t")

    // Driver-specific types
    generate!("vr::DriverPose_t")
    generate!("vr::DriverPoseQuaternion_t")
    generate!("vr::TrackedDeviceDriverInfo_t")

    // Enums
    generate!("vr::EVREye")
    generate!("vr::ETextureType")
    generate!("vr::EColorSpace")
    generate!("vr::ETrackingResult")
    generate!("vr::ETrackedDeviceClass")
    generate!("vr::ETrackedControllerRole")
    generate!("vr::EVRInitError")
    generate!("vr::EVRSettingsError")
    generate!("vr::ETrackedPropertyError")
    generate!("vr::EPropertyWriteType")
    generate!("vr::EVRScalarType")

    // Core driver interfaces
    generate!("vr::ITrackedDeviceServerDriver")
    generate!("vr::IVRDisplayComponent")
    //generate!("vr::IVRDriverDirectModeComponent")
    generate!("vr::IVRCameraComponent")
    generate!("vr::ICameraVideoSinkCallback")
    generate!("vr::IVRDriverContext")
    generate!("vr::IServerTrackedDeviceProvider")
    generate!("vr::IVRWatchdogProvider")
    generate!("vr::IVRCompositorPluginProvider")

    // Helper/utility interfaces
    generate!("vr::IVRProperties")
    generate!("vr::IVRDriverInput")
    generate!("vr::IVRDriverLog")
    generate!("vr::IVRServerDriverHost")
    generate!("vr::IVRCompositorDriverHost")
    generate!("vr::IVRWatchdogHost")
    generate!("vr::IVRVirtualDisplay")
    generate!("vr::IVRResources")
    generate!("vr::IVRIOBuffer")
    generate!("vr::IVRSettings")

    // Handles and IDs
    generate!("vr::TrackedDeviceIndex_t")
    generate!("vr::DriverId_t")
    generate!("vr::SpatialAnchorHandle_t")
    generate!("vr::SharedTextureHandle_t")
    generate!("vr::PropertyContainerHandle_t")
    generate!("vr::DriverHandle_t")
    generate!("vr::VRInputComponentHandle_t")

    // Property system types
    generate!("vr::PropertyRead_t")
    generate!("vr::PropertyWrite_t")

    // Constants (these might not work with generate!, may need to define manually)
    // generate!("vr::k_unMaxTrackedDeviceCount")
    // generate!("vr::k_unTrackedDeviceIndex_Hmd")

    safety!(unsafe_ffi)
}

// Re-export everything from the generated bindings
pub use ffi::*;

pub mod bridge;
pub use bridge::*;

// Manual constants that autocxx can't generate
pub mod constants {
    // Device tracking constants
    pub const K_UN_MAX_TRACKED_DEVICE_COUNT: u32 = 64;
    pub const K_UN_TRACKED_DEVICE_INDEX_HMD: u32 = 0;
    pub const K_UN_TRACKED_DEVICE_INDEX_OTHER: u32 = 0xFFFFFFFE;
    pub const K_UN_TRACKED_DEVICE_INDEX_INVALID: u32 = 0xFFFFFFFF;

    // Driver constants
    pub const K_N_DRIVER_NONE: u32 = 0xFFFFFFFF;
    pub const K_UN_MAX_DRIVER_DEBUG_RESPONSE_SIZE: u32 = 32768;
}

pub mod direct_mode {
    use crate::ffi::vr;

    #[repr(C)]
    pub struct SwapTextureSetDesc {
        pub width: u32,
        pub height: u32,
        pub format: u32,
        pub sample_count: u32,
    }

    #[repr(C)]
    pub struct SwapTextureSet {
        pub shared_texture_handles: [vr::SharedTextureHandle_t; 3],
        pub texture_flags: u32,
    }

    #[repr(C)]
    pub struct SubmitLayerPerEye {
        /// Shared texture handle for color
        pub texture: vr::SharedTextureHandle_t,
        /// Shared texture handle for depth (optional)
        pub depth_texture: vr::SharedTextureHandle_t,
        /// Valid region of provided texture (and depth)
        pub bounds: vr::VRTextureBounds_t,
        /// Projection matrix used to render the depth buffer
        pub projection: vr::HmdMatrix44_t,
        /// HMD pose used to render this layer
        pub hmd_pose: vr::HmdMatrix34_t,
        /// Time in seconds from now that hmd_pose was predicted to
        pub hmd_pose_prediction_time_in_seconds_from_now: f32,
    }

    #[repr(C)]
    pub struct Throttling {
        pub frames_to_throttle: u32,
        pub additional_frames_to_predict: u32,
    }

    #[repr(C)]
    pub struct DriverDirectMode_FrameTiming {
        /// Set to sizeof(DriverDirectMode_FrameTiming) - for versioning
        pub size: u32,
        /// Number of times frame was presented
        pub num_frame_presents: u32,
        /// Number of times frame was presented on a vsync other than originally predicted
        pub num_mis_presented: u32,
        /// Number of additional times previous frame was scanned out (compositor missed vsync)
        pub num_dropped_frames: u32,
        /// Reprojection flags (VRCompositor_ReprojectionMotion_* constants)
        pub reprojection_flags: u32,
    }

    // Reprojection constants
    pub const VR_COMPOSITOR_REPROJECTION_MOTION_ENABLED: u32 = 0x100;
    pub const VR_COMPOSITOR_REPROJECTION_MOTION_FORCED_ON: u32 = 0x200;

    /// Trait representing the IVRDriverDirectModeComponent interface
    pub trait IVRDriverDirectModeComponent {
        fn create_swap_texture_set(
            &mut self,
            pid: u32,
            desc: &SwapTextureSetDesc,
            out_set: &mut SwapTextureSet,
        );
        fn destroy_swap_texture_set(&mut self, shared_texture_handle: vr::SharedTextureHandle_t);
        fn destroy_all_swap_texture_sets(&mut self, pid: u32);
        fn get_next_swap_texture_set_index(
            &mut self,
            shared_handles: [vr::SharedTextureHandle_t; 2],
            indices: &mut [u32; 2],
        );

        /// Submit layer data for both eyes
        fn submit_layer(&mut self, per_eye: &[SubmitLayerPerEye; 2]);

        /// Present all submitted layers
        fn present(&mut self, sync_texture: vr::SharedTextureHandle_t);

        /// Called after Present for additional timing control
        fn post_present(&mut self, throttling: Option<&Throttling>);

        /// Get frame timing stats - has default implementation
        fn get_frame_timing(&mut self, frame_timing: &mut DriverDirectMode_FrameTiming) {
            // Default: clear reprojection flags to avoid being interpreted as throttling
            frame_timing.reprojection_flags = 0;
        }
    }

    /// Interface version string
    pub const INTERFACE_VERSION: &str = "IVRDriverDirectModeComponent_009";
}
