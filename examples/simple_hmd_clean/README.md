# Simple HMD Clean Example

A clean, minimal OpenVR HMD driver implementation using the high-level `openvr-driver` library. This example demonstrates how to create a virtual HMD driver for SteamVR without dealing with low-level FFI and vtable boilerplate.

## Overview

This example creates a simple virtual HMD that:
- Appears as a connected headset in SteamVR
- Reports basic display configuration (1920x1080 @ 90Hz)
- Provides animated pose data (slowly rotating)
- Uses the high-level safe Rust API from `openvr-driver`

## Project Structure

```
simple_hmd_clean/
├── src/
│   └── lib.rs              # Driver implementation (~340 lines)
├── driver/                 # OpenVR driver structure
│   ├── driver.vrdrivermanifest
│   ├── bin/               # Platform-specific binaries (generated)
│   │   └── linux64/       # Linux x64 binaries
│   │       └── driver_simple_hmd_clean.so
│   └── resources/
│       └── settings/
│           └── default.vrsettings
├── build_and_deploy.sh    # Build and deployment script
├── Cargo.toml
└── README.md
```

## Building

### Prerequisites

- Rust toolchain (stable)
- SteamVR installed
- OpenVR headers (included in the project)

### Build Commands

```bash
# Build the driver (debug mode)
cargo build --package simple_hmd_clean

# Build the driver (release mode)
cargo build --package simple_hmd_clean --release

# Or use the build script
./build_and_deploy.sh
```

## Deployment

The driver needs to be deployed in the correct structure for OpenVR to load it:

```bash
# Build and deploy to driver directory
./build_and_deploy.sh --deploy

# Build, deploy, and register with SteamVR
./build_and_deploy.sh --register
```

### Manual Registration

If you prefer to manually register the driver:

```bash
# Linux
~/.steam/steam/steamapps/common/SteamVR/bin/linux64/vrpathreg adddriver /path/to/driver

# Windows
"C:\Program Files (x86)\Steam\steamapps\common\SteamVR\bin\win64\vrpathreg.exe" adddriver C:\path\to\driver

# macOS
/Applications/SteamVR.app/bin/osx32/vrpathreg adddriver /path/to/driver
```

## Code Structure

The example demonstrates clean separation between driver logic and OpenVR infrastructure:

```rust
// Main provider - entry point for the driver
pub struct SimpleHmdProvider {
    devices: Vec<Arc<HmdDeviceWrapper>>,
}

impl ServerTrackedDeviceProvider for SimpleHmdProvider {
    fn init(&mut self, context: &mut DriverContext) -> DriverResult<()> { ... }
    fn cleanup(&mut self) { ... }
    fn run_frame(&mut self) { ... }
    // ...
}

// Device implementation with interior mutability
pub struct HmdDeviceWrapper(Arc<Mutex<HmdDeviceInner>>);

impl TrackedDeviceServerDriver for HmdDeviceWrapper {
    fn activate(&mut self, device_index: u32) -> DriverResult<()> { ... }
    fn get_pose(&self) -> DriverPose { ... }
    fn get_component(&self, component_name: &str) -> ComponentResult { ... }
    // ...
}

// Display component for HMD configuration
impl DisplayComponent for DisplayComponentImpl {
    fn get_window_bounds(&self) -> (i32, i32, i32, i32) { ... }
    fn get_recommended_render_target_size(&self) -> (u32, u32) { ... }
    // ...
}

// Entry point - just one line!
openvr_driver_entry!(SimpleHmdProvider);
```

## Key Features

### Clean API
- No vtable boilerplate
- No unsafe FFI code in the example
- Simple trait implementations
- Type-safe error handling

### Interior Mutability Pattern
The example uses `Arc<Mutex<...>>` for thread-safe access to device state, as OpenVR may call driver methods from multiple threads.

### Configuration
Driver settings are loaded from `driver/resources/settings/default.vrsettings`:
- Display resolution and refresh rate
- IPD (interpupillary distance)
- Serial number and model information

## Testing

1. Build and register the driver:
   ```bash
   ./build_and_deploy.sh --register
   ```

2. Start SteamVR

3. Check SteamVR Settings → Startup/Shutdown → Manage Add-ons
   - The "simple_hmd_clean" driver should appear in the list

4. The virtual HMD should appear as a connected device

5. To unregister:
   ```bash
   # Linux
   ~/.steam/steam/steamapps/common/SteamVR/bin/linux64/vrpathreg removedriver /path/to/driver
   ```

## Troubleshooting

### Driver not loading
- Check SteamVR logs: `~/.steam/steam/logs/vrserver.txt`
- Verify the library name matches the pattern: `driver_simple_hmd_clean.{so,dll,dylib}`
- Ensure the driver manifest is valid JSON

### Properties not being set
- The current implementation logs property operations instead of actually setting them
- This is a known limitation that will be addressed in future versions

### Device not appearing
- Check that the driver is registered: `vrpathreg show`
- Verify the driver is enabled in SteamVR settings
- Check for errors in the SteamVR console

## Comparison with Original Example

| Aspect | Original (simplehmd) | Clean (simple_hmd_clean) |
|--------|---------------------|--------------------------|
| Lines of Code | ~1000+ | ~340 |
| Vtable Boilerplate | ~700 lines | 0 lines |
| Unsafe Code | Extensive | None (in example) |
| Complexity | High (FFI details) | Low (business logic only) |
| Maintainability | Difficult | Easy |

## Future Improvements

- [ ] Implement actual property writing through the context
- [ ] Add controller support
- [ ] Implement camera component for passthrough
- [ ] Add haptic feedback support
- [ ] Create more complex tracking patterns
- [ ] Add configuration hot-reloading

## License

This example is part of the openvr-driver-rs project and is available under the same license terms (MIT OR Apache-2.0).