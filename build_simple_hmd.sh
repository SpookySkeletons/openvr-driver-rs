#!/bin/bash
set -e

echo "🦀 Building Simple HMD Driver Package..."

# Build the Rust driver
echo "Building driver binary..."
cargo build --example simple_hmd --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

# Create driver package structure
PACKAGE_DIR="target/simple_hmd_driver_package"
echo "📦 Creating package: $PACKAGE_DIR"

mkdir -p "$PACKAGE_DIR"/{bin/linux64,bin/win64,resources/{settings,icons,rendermodels/simple_hmd}}
cp target/release/examples/libsimple_hmd.so "$PACKAGE_DIR/bin/linux64/driver_simple_hmd_rust.so"

echo "📄 Generating driver configuration..."

# Driver manifest
cat > "$PACKAGE_DIR/driver.vrdrivermanifest" << 'EOF'
{
    "name": "simple_hmd_rust",
    "directory": "",
    "resourceOnly": false,
    "binary": "driver_simple_hmd_rust",
    "tracked_device_driver": "driver_simple_hmd_rust",
    "description": "Simple HMD driver written in Rust",
    "author": "OpenVR Rust Bindings Project",
    "version": "0.1.0",
    "minimum_steamvr_version": "1.0.0"
}
EOF

# Driver resources
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

# Default settings
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

# Placeholder assets
echo "Simple HMD Status Icon - Replace with PNG" > "$PACKAGE_DIR/resources/icons/simple_hmd_status.png"
echo "Simple HMD 3D Model - Replace with OBJ" > "$PACKAGE_DIR/resources/rendermodels/simple_hmd/simple_hmd.obj"

echo "🔧 Creating installation script..."

# Install script
cat > "$PACKAGE_DIR/install.sh" << 'EOF'
#!/bin/bash
echo "🦀 Installing Simple HMD Rust Driver..."

# Locate SteamVR installation
STEAMVR_DRIVERS=""
if [ -d "$HOME/.steam/steam/steamapps/common/SteamVR/drivers" ]; then
    STEAMVR_DRIVERS="$HOME/.steam/steam/steamapps/common/SteamVR/drivers"
elif [ -d "$HOME/.local/share/Steam/steamapps/common/SteamVR/drivers" ]; then
    STEAMVR_DRIVERS="$HOME/.local/share/Steam/steamapps/common/SteamVR/drivers"
fi

if [ -n "$STEAMVR_DRIVERS" ]; then
    echo "🎯 Found SteamVR: $STEAMVR_DRIVERS"
    cp -r . "$STEAMVR_DRIVERS/simple_hmd_rust"
    echo "✅ Driver installed successfully!"
    echo ""
    echo "📋 To test:"
    echo "  1. Start SteamVR"
    echo "  2. Monitor logs: tail -f ~/.steam/steam/logs/vrserver.txt | grep -i simple"
    echo "  3. Look for 'HmdDriverFactory' messages"
else
    echo "❌ SteamVR not found"
    echo "   Manual install: Copy this folder to [SteamVR]/drivers/simple_hmd_rust"
fi
EOF

chmod +x "$PACKAGE_DIR/install.sh"

# Package documentation
cat > "$PACKAGE_DIR/README.md" << 'EOF'
# Simple HMD Rust Driver

Minimal OpenVR driver written in Rust for testing the bindings.

## Installation
Run `./install.sh` to install to SteamVR automatically.

## Testing
1. Start SteamVR
2. Check logs: `tail -f ~/.steam/steam/logs/vrserver.txt | grep -i simple`
3. Look for "HmdDriverFactory" messages

## Status
- ✅ Compiles and loads into SteamVR
- ✅ Shows initialization messages
- ✅ Demonstrates basic driver structure
EOF

echo ""
echo "🎉 Driver package complete!"
echo "📦 Location: $PACKAGE_DIR"
echo ""
echo "🚀 Next steps:"
echo "   cd $PACKAGE_DIR && ./install.sh"
echo ""
echo "✅ Package includes:"
echo "   - Driver binary and manifest"
echo "   - Default settings and resources"
echo "   - Installation script"
