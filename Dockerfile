# --- Stage 1: Build the Application ---
# Use the standard, full-featured Rust image as a build environment.
FROM rust:latest AS builder

# Install the system libraries and development headers required to compile the GUI.
RUN apt-get update && apt-get install -y \
    pkg-config \
    libx11-dev \
    libxcursor-dev \
    libxrandr-dev \
    libxinerama-dev \
    libxi-dev \
    libgl1-mesa-dev \
    libwayland-dev \
    libxkbcommon-dev \
    libfontconfig1-dev \
    --no-install-recommends \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/chess-engine

# Copy the entire project into the build container (respecting .dockerignore).
COPY . .

# Build the application in release mode for optimal performance.
RUN cargo build --release -p chess-gui --bin best


# --- Stage 2: Create the Final Runtime Image ---
# Use the full debian:bookworm image. It is larger but contains all necessary
# low-level libraries that the slim version lacks, which is critical for graphics.
FROM debian:bookworm

# Install only the runtime libraries required for software rendering.
RUN apt-get update && apt-get install -y \
    libx11-6 \
    libxcursor1 \
    libxrandr2 \
    libxinerama1 \
    libxi6 \
    libgl1 \
    libgl1-mesa-dri \
    libglx-mesa0 \
    libwayland-client0 \
    libxkbcommon0 \
    libxkbcommon-x11-0 \
    libfontconfig1 \
    ca-certificates \
    --no-install-recommends \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary and the required assets from the builder stage.
COPY --from=builder /usr/src/chess-engine/target/release/best .
COPY --from=builder /usr/src/chess-engine/assets ./assets

# --- CRITICAL ENVIRONMENT VARIABLES FOR SOFTWARE RENDERING ---
# These force the container to use CPU-based rendering, making it independent of host GPU drivers.
ENV LIBGL_ALWAYS_SOFTWARE=true
ENV WGPU_BACKEND=gl

# The command to execute when the container starts.
CMD ["./best"]