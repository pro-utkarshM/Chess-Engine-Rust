#!/bin/bash
#
# A reliable script to build and run the Rust Chess Engine GUI locally.
# This script handles dependency checks and uses the correct flags for compatibility.
#

set -e

echo "--- Rust Chess Engine GUI Runner ---"

# --- 1. Dependency Check ---
# Check for required system libraries and prompt for installation if missing.
echo "[1/3] Checking for required system libraries..."
LIBS_TO_CHECK="pkg-config libx11-dev libxcursor-dev libxrandr-dev libxinerama-dev libxi-dev libgl1-mesa-dev libwayland-dev libxkbcommon-dev"
MISSING_LIBS=""
for LIB in $LIBS_TO_CHECK; do
    if ! dpkg -s "$LIB" &> /dev/null; then
        MISSING_LIBS="$MISSING_LIBS $LIB"
    fi
done

if [ ! -z "$MISSING_LIBS" ]; then
    echo "WARNING: The following required libraries are missing:$MISSING_LIBS"
    read -p "Would you like to try and install them now? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        sudo apt-get update
        sudo apt-get install -y $MISSING_LIBS
    else
        echo "Installation cancelled. The build may fail."
    fi
else
    echo "All system libraries are present."
fi

# --- 2. Build the Application ---
echo "[2/3] Building the application with Cargo..."
cargo build -p chess-gui --bin best
echo "Build complete."

# --- 3. Run the Application ---
# Use the WGPU_BACKEND=gl flag to ensure maximum graphics compatibility on Linux.
echo "[3/3] Starting the chess application..."
echo "----------------------------------------"
WGPU_BACKEND=gl ./target/debug/best