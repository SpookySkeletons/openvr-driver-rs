# Simple HMD Driver Example

A minimal OpenVR HMD driver implementation in pure Rust that demonstrates the basic structure and lifecycle of an OpenVR driver.

## Overview

This example implements a basic virtual HMD that:
- Reports as a simple HMD device to SteamVR
- Provides a basic pose that slowly rotates
- Demonstrates the minimal interface implementation required for a functional driver

## Building

### Quick Build & Deploy

Use the provided build script for easy deployment:

```bash
# Build only
./examples/simplehmd/build_and_deploy.sh

# Build and deploy to driver directory
./examples/simplehmd/build_and_deploy.sh --deploy

# Build, deploy, and register with SteamVR
./examples/simplehmd/build_and_deploy.sh --register

# Build in debug mode
./examples/simplehmd/build_and_deploy.sh --debug
```

### Manual Build

From the repository root:

```bash
cargo build --package simplehmd --release
```

This will create a shared library at:
- Linux: `target/release/libsimplehmd.so`
- Windows: `target/release/simplehmd.dll`
- macOS: `target/release/libsimplehmd.dylib`

## Installation

1. Copy the built library to the driver's bin directory:
   ```bash
   cp target/release/libsimplehmd.so examples/simplehmd/driver/bin/linux64/driver_simplehmd.so
   ```
   (Adjust paths and extensions for your platform)

2. Register the driver with SteamVR:
   ```bash
   # Linux
   ~/.steam/steam/steamapps/common/SteamVR/bin/linux64/vrpathreg adddriver /path/to/openvr-driver-rs/examples/simplehmd/driver
   
   # Windows
   "C:\Program Files (x86)\Steam\steamapps\common\SteamVR\bin\win64\vrpathreg.exe" adddriver C:\path\to\openvr-driver-rs\examples\simplehmd\driver
   ```

3. Start SteamVR - the driver should load automatically

## Driver Structure

### Key Components

- **HmdDevice**: Represents the virtual HMD device
  - Manages device state and pose
  - Handles activation/deactivation
  - Provides display component configuration

- **SimpleHmdProvider**: The main driver provider
  - Manages driver lifecycle
  - Creates and registers devices
  - Handles frame updates

- **DisplayComponent**: Display configuration
  - Window and render dimensions
  - Display properties

### Entry Points

The driver exports these functions that OpenVR calls:

- `HmdDriverFactory`: Main entry point that returns interface implementations
- `HmdDriverFactory_GetInterfaceVersions`: Returns supported interface versions

## Configuration

Settings are stored in `driver/resources/settings/default.vrsettings`:

```json
{
    "driver_simplehmd": {
        "serial_number": "SIMPLEHMD_001",
        "model_number": "SimpleHMD_Model_v1"
    },
    "simplehmd_display": {
        "window_width": 1920,
        "window_height": 1080,
        "render_width": 1920,
        "render_height": 1080,
        "display_frequency": 90.0
    }
}
```

## Development Notes

### Current Implementation Status

✅ **Fully Implemented:**
- Complete driver structure with modular design
- HMD device with pose tracking
- Pose generation (simple rotation animation)
- Driver manifest and settings
- **Vtable generation and binding system**
- **Proper interface implementation for IServerTrackedDeviceProvider**
- **Proper interface implementation for ITrackedDeviceServerDriver**
- **HmdDriverFactory returns functional vtable pointers**
- Build and deployment scripts

⚠️ **Partially Implemented:**
- Device registration with OpenVR host (structure in place, needs testing)
- Display component (structure ready, vtable not yet returned)
- Full device property configuration

❌ **Not Yet Implemented:**
- Input component handling
- Proper display distortion calculations
- Full integration testing with SteamVR

### Testing

Run the unit tests:
```bash
cargo test --package simplehmd
```

### Debugging

Enable debug output by setting environment variables:
```bash
export RUST_LOG=debug
export RUST_BACKTRACE=1
```

Check SteamVR logs at:
- Linux: `~/.steam/steam/logs/vrserver.txt`
- Windows: `%LOCALAPPDATA%\openvr\logs\vrserver.txt`

## Next Steps

The driver infrastructure is now complete! To finish the implementation:

1. ✅ ~~Complete the vtable implementation in the bindings crate~~
2. ✅ ~~Wire up the `HmdDriverFactory` to return proper interface pointers~~
3. Test device registration with `IVRServerDriverHost::TrackedDeviceAdded`
4. Complete display component vtable return in `GetComponent`
5. Add controller device support
6. Test with actual SteamVR runtime
7. Add proper error handling and logging

## License

This example is part of the openvr-driver-rs project and shares its licensing.