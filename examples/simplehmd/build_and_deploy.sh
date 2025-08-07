#!/bin/bash

# Build and Deploy Script for SimpleHMD Driver
# This script builds the SimpleHMD driver and optionally deploys it to SteamVR

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
DRIVER_NAME="simplehmd"
PROJECT_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
DRIVER_DIR="$PROJECT_ROOT/examples/simplehmd/driver"

echo -e "${GREEN}SimpleHMD Driver Build & Deploy Script${NC}"
echo "----------------------------------------"

# Detect platform
OS="$(uname -s)"
case "${OS}" in
    Linux*)
        PLATFORM="linux64"
        LIB_PREFIX="lib"
        LIB_EXT="so"
        STEAMVR_ROOT="$HOME/.steam/steam/steamapps/common/SteamVR"
        ;;
    Darwin*)
        PLATFORM="osx"
        LIB_PREFIX="lib"
        LIB_EXT="dylib"
        STEAMVR_ROOT="/Applications/SteamVR.app"
        ;;
    MINGW*|CYGWIN*|MSYS*)
        PLATFORM="win64"
        LIB_PREFIX=""
        LIB_EXT="dll"
        STEAMVR_ROOT="C:/Program Files (x86)/Steam/steamapps/common/SteamVR"
        ;;
    *)
        echo -e "${RED}Error: Unsupported platform: ${OS}${NC}"
        exit 1
        ;;
esac

echo "Platform detected: $PLATFORM"
echo "Project root: $PROJECT_ROOT"

# Parse command line arguments
BUILD_TYPE="release"
DEPLOY=false
REGISTER=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            BUILD_TYPE="debug"
            shift
            ;;
        --deploy)
            DEPLOY=true
            shift
            ;;
        --register)
            REGISTER=true
            DEPLOY=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --debug      Build in debug mode (default: release)"
            echo "  --deploy     Deploy the driver to the driver directory"
            echo "  --register   Register the driver with SteamVR (implies --deploy)"
            echo "  --help       Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Build the driver
echo -e "\n${YELLOW}Building driver in $BUILD_TYPE mode...${NC}"
cd "$PROJECT_ROOT"

if [ "$BUILD_TYPE" = "release" ]; then
    cargo build --package "$DRIVER_NAME" --release
    BUILD_DIR="target/release"
else
    cargo build --package "$DRIVER_NAME"
    BUILD_DIR="target/debug"
fi

# Check if build succeeded
SOURCE_LIB="$PROJECT_ROOT/$BUILD_DIR/${LIB_PREFIX}${DRIVER_NAME}.${LIB_EXT}"
if [ ! -f "$SOURCE_LIB" ]; then
    echo -e "${RED}Error: Build failed. Library not found at: $SOURCE_LIB${NC}"
    exit 1
fi

echo -e "${GREEN}Build successful!${NC}"
echo "Built library: $SOURCE_LIB"

# Deploy if requested
if [ "$DEPLOY" = true ]; then
    echo -e "\n${YELLOW}Deploying driver...${NC}"

    # Create bin directory for the platform
    BIN_DIR="$DRIVER_DIR/bin/$PLATFORM"
    mkdir -p "$BIN_DIR"

    # Copy the library with the correct name
    TARGET_LIB="$BIN_DIR/driver_${DRIVER_NAME}.${LIB_EXT}"
    echo "Copying to: $TARGET_LIB"
    cp "$SOURCE_LIB" "$TARGET_LIB"

    # Copy driver manifest if it doesn't exist
    if [ ! -f "$DRIVER_DIR/driver.vrdrivermanifest" ]; then
        echo -e "${YELLOW}Warning: driver.vrdrivermanifest not found${NC}"
    fi

    # Copy resources if they exist
    if [ -d "$DRIVER_DIR/resources" ]; then
        echo "Resources directory found"
    else
        echo -e "${YELLOW}Warning: resources directory not found${NC}"
    fi

    echo -e "${GREEN}Driver deployed successfully!${NC}"
fi

# Register with SteamVR if requested
if [ "$REGISTER" = true ]; then
    echo -e "\n${YELLOW}Registering driver with SteamVR...${NC}"

    # Find vrpathreg executable
    VRPATHREG=""
    if [ "$PLATFORM" = "linux64" ]; then
        VRPATHREG="$STEAMVR_ROOT/bin/linux64/vrpathreg"
    elif [ "$PLATFORM" = "win64" ]; then
        VRPATHREG="$STEAMVR_ROOT/bin/win64/vrpathreg.exe"
    elif [ "$PLATFORM" = "osx" ]; then
        VRPATHREG="$STEAMVR_ROOT/bin/osx32/vrpathreg"
    fi

    if [ ! -f "$VRPATHREG" ]; then
        echo -e "${RED}Error: vrpathreg not found at: $VRPATHREG${NC}"
        echo "Please make sure SteamVR is installed"
        exit 1
    fi

    # Register the driver
    echo "Registering driver path: $DRIVER_DIR"
    "$VRPATHREG" adddriver "$DRIVER_DIR"

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}Driver registered successfully!${NC}"
        echo ""
        echo "To verify registration, run:"
        echo "  $VRPATHREG show"
        echo ""
        echo "To remove the driver later, run:"
        echo "  $VRPATHREG removedriver \"$DRIVER_DIR\""
    else
        echo -e "${RED}Error: Failed to register driver${NC}"
        exit 1
    fi
fi

echo -e "\n${GREEN}Done!${NC}"

if [ "$DEPLOY" = false ]; then
    echo ""
    echo "To deploy the driver, run:"
    echo "  $0 --deploy"
    echo ""
    echo "To deploy and register with SteamVR, run:"
    echo "  $0 --register"
fi
