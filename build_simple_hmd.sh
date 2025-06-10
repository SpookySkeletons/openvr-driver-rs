#!/bin/bash
set -e

echo "🦀 Building Simple HMD driver with complete package..."

# Build the example first
cargo build --example simple_hmd --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Driver binary built successfully!"

# Create driver package
PACKAGE_DIR="target/simple_hmd_driver_package"
echo "📦 Creating driver package at: $PACKAGE_DIR"

# Create directory structure
mkdir -p "$PACKAGE_DIR"/{bin/linux64,bin/win64,resources/{settings,icons,rendermodels/simple_hmd}}

# Copy the built binary
cp target/release/examples/libsimple_hmd.so "$PACKAGE_DIR/bin/linux64/driver_simple_hmd_rust.so"
echo "📄 Copied driver binary"

# Generate driver manifest
cat > "$PACKAGE_DIR/driver.vrdrivermanifest" << 'EOF'
{
    "name": "simple_hmd_rust",
    "directory": "",
    "resourceOnly": false,
    "binary": "driver_simple_hmd_rust",
    "tracked_device_driver": "driver_simple_hmd_rust",
    "description": "Simple HMD driver written in Rust - OpenVR bindings test",
    "author": "OpenVR Rust Bindings Project",
    "version": "0.1.0",
    "minimum_steamvr_version": "1.0.0"
}
EOF

# Generate driver resources
cat > "$PACKAGE_DIR/resources/driver.vrresources" << 'EOF'
{
    "jsonid": "vrresources",
    "statusicons": [
        {
            "id": "simple_hmd_rust_status",
            "filename": "/icons/simple_hmd_status.png"
        }
    ],
    "tracked_devices": [
        {
            "simple_hmd_rust_hmd": {
                "filename": "/rendermodels/simple_hmd/simple_hmd",
                "visible_in_first_person": false,
                "controller_type": "simple_hmd_rust",
                "generic_tracker": false
            }
        }
    ]
}
EOF

# Generate default settings
cat > "$PACKAGE_DIR/resources/settings/default.vrsettings" << 'EOF'
{
    "driver_simple_hmd_rust": {
        "enable": true,
        "loadPriority": 0,
        "blocked_by_safe_mode": false,
        "hmd_tracking_system": "simple_hmd_rust",
        "hmd_model_name": "Simple HMD Rust",
        "hmd_manufacturer": "OpenVR Rust Bindings",
        "hmd_display_frequency": 90.0,
        "hmd_ipd": 0.063,
        "hmd_display_width": 1920,
        "hmd_display_height": 1080,
        "hmd_render_width": 1512,
        "hmd_render_height": 1680,
        "debug_logging": true
    }
}
EOF

# Create placeholder assets
echo "Simple HMD Status Icon - Replace with PNG" > "$PACKAGE_DIR/resources/icons/simple_hmd_status.png"
echo "Simple HMD 3D Model - Replace with OBJ" > "$PACKAGE_DIR/resources/rendermodels/simple_hmd/simple_hmd.obj"

# Create install script
cat > "$PACKAGE_DIR/install.sh" << 'EOF'
#!/bin/bash
echo "🦀 Installing Simple HMD Rust Driver..."

# Find SteamVR installation
STEAMVR_DRIVERS=""
if [ -d "$HOME/.steam/steam/steamapps/common/SteamVR/drivers" ]; then
    STEAMVR_DRIVERS="$HOME/.steam/steam/steamapps/common/SteamVR/drivers"
elif [ -d "$HOME/.local/share/Steam/steamapps/common/SteamVR/drivers" ]; then
    STEAMVR_DRIVERS="$HOME/.local/share/Steam/steamapps/common/SteamVR/drivers"
fi

if [ -n "$STEAMVR_DRIVERS" ]; then
    echo "🎯 Found SteamVR at: $STEAMVR_DRIVERS"
    echo "Installing driver..."
    cp -r . "$STEAMVR_DRIVERS/simple_hmd_rust"
    echo "✅ Driver installed to SteamVR!"
    echo ""
    echo "⚠️  WARNING: This will crash SteamVR until the C++ bridge is implemented!"
    echo "             But you should see 'HmdDriverFactory' messages in the logs first."
    echo ""
    echo "📋 To test:"
    echo "  1. Start SteamVR"
    echo "  2. Watch logs: tail -f ~/.steam/steam/logs/vrserver.txt | grep -i simple"
    echo "  3. Look for 'HmdDriverFactory: Requested interface' messages"
else
    echo "❌ SteamVR not found. Manual install:"
    echo "   Copy this entire folder to: [SteamVR]/drivers/simple_hmd_rust"
fi
EOF

chmod +x "$PACKAGE_DIR/install.sh"

# Create README
cat > "$PACKAGE_DIR/README.md" << 'EOF'
# Simple HMD Rust Driver

Test OpenVR driver written in Rust.

## Status
- ✅ Compiles and loads into SteamVR
- ✅ Shows initialization messages in logs
- ❌ Missing C++ bridge (crashes when SteamVR calls virtual functions)

## Install
Run `./install.sh`

## Test
1. Start SteamVR
2. Check logs: `tail -f ~/.steam/steam/logs/vrserver.txt | grep -i simple`
3. Look for "HmdDriverFactory" messages - this proves the driver loads!

Expected: Driver loads, shows messages, then crashes. This is normal without the bridge.
EOF

echo ""
echo "🎉 Complete driver package ready!"
echo "📍 Location: $PACKAGE_DIR"
echo ""
echo "🚀 To install and test:"
echo "   cd $PACKAGE_DIR && ./install.sh"
echo ""
echo "⚠️  Expected behavior: Driver loads, shows logs, then crashes (missing bridge)"
EOF

chmod +x build_simple_hmd.sh
