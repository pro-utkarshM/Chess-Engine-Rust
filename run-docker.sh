#!/bin/bash
# A script to reliably build and run the Rust Chess Engine GUI in a Docker container.

set -e

IMAGE_NAME="chess-gui"

# --- Forceful Cleanup Phase ---
# First, remove any existing image with the same name to guarantee a fresh build.
echo "Forcefully removing old Docker image if it exists..."
docker rmi -f ${IMAGE_NAME} 2>/dev/null || true
echo "Cleanup complete."


# --- Build Phase ---
echo "Building Docker image '${IMAGE_NAME}' from scratch..."
docker build -t ${IMAGE_NAME} .
echo "Build complete."


# --- Permissions Phase ---
echo "Setting X11 permissions..."
xhost +local:docker &> /dev/null
echo "Permissions set."


# --- Run Phase ---
# This is the definitive command to run a GUI application with software rendering.
echo "Starting the application..."
# --- Run Phase ---
echo "Starting the application..."
docker run -it --rm \
    --env="DISPLAY" \
    --volume="/tmp/.X11-unix:/tmp/.X11-unix:rw" \
    --device=/dev/dri:/dev/dri \
    ${IMAGE_NAME}

# --- Final Cleanup Phase ---
echo "Cleaning up X11 permissions..."
xhost -local:docker &> /dev/null
echo "Done."