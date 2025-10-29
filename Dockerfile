# Use Ubuntu as base for building
FROM ubuntu:24.04 AS builder
ENV DEBIAN_FRONTEND=noninteractive

# Install required packages (curl, git, build tools, mingw for cross compile)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      ca-certificates curl git build-essential pkg-config python3 cmake \
      mingw-w64 libssl-dev unzip xz-utils && \
    rm -rf /var/lib/apt/lists/*

# Install Node.js 22 (required for vite/svelte/tauri frontend build)
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && \
    apt-get install -y nodejs && \
    node --version && npm --version

# Install rustup and stable toolchain
# Note: this will install to /root/.cargo by default in the container
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

# Configure build target. Default to MSVC target to match GitLab workflow
ARG BUILD_TARGET=x86_64-pc-windows-msvc
ENV BUILD_TARGET=${BUILD_TARGET}

# Add the Windows GNU target (kept as a fallback) and the MSVC target used by cargo-xwin
RUN rustup target add x86_64-pc-windows-gnu || true && \
    rustup target add x86_64-pc-windows-msvc || true

# Configure cross toolchain environment for mingw (GNU) builds
ENV CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
ENV CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
ENV CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++
ENV PKG_CONFIG_ALLOW_CROSS=1

# Install additional tools needed to reproduce the GitLab job locally (nsis, wine, llvm, bun)
RUN apt-get update && apt-get install -y --no-install-recommends \
      wget unzip nsis wine64 wine32 lld llvm clang && \
    rm -rf /var/lib/apt/lists/* || true

# Install bun (optional fast JS runtime/installer)
RUN curl -fsSL https://bun.sh/install | bash || true

# Install cargo-xwin to allow building MSVC artifacts from Linux (mirrors GitLab job)
ENV PATH="/root/.cargo/bin:${PATH}"
RUN . /root/.cargo/env && cargo install --locked cargo-xwin || true

# Set working directory and copy project files
WORKDIR /workdir
# Copy everything; in a real repo you might want a .dockerignore to speed things up
COPY . .


# Install node deps (including dev deps for tauri) and build frontend
RUN if [ -f package-lock.json ]; then npm ci --no-audit --prefer-offline; else npm install --no-audit --prefer-offline; fi
RUN npm run build

# Build the Rust/Tauri app for the requested Windows target
WORKDIR /workdir
# Use cargo-xwin for MSVC builds (mirrors GitLab job). Fall back to plain cargo for GNU target.
RUN . /root/.cargo/env || true && \
        if [ "$BUILD_TARGET" = "x86_64-pc-windows-msvc" ]; then \
            # Run Tauri build from project root; tauri will drive cargo-xwin for the Rust crate
            npm run tauri build -- --runner cargo-xwin --target "$BUILD_TARGET"; \
        else \
            # For GNU target build the Rust crate directly
            (cd src-tauri && cargo build --release --target "$BUILD_TARGET"); \
        fi

# Stage: collect build artifacts
RUN mkdir -p /output
# copy executables, installer bundles and the web build into /output
RUN mkdir -p /output
RUN cp -v src-tauri/target/${BUILD_TARGET}/release/*.exe /output/ 2>/dev/null || true
RUN cp -v src-tauri/target/${BUILD_TARGET}/release/*.dll /output/ 2>/dev/null || true
RUN cp -v src-tauri/target/${BUILD_TARGET}/release/bundle/nsis/*.exe /output/ 2>/dev/null || true
RUN cp -v -r /workdir/build /output/web || true

# Final tiny image to export results (makes it easy to copy them out)
FROM scratch AS export
COPY --from=builder /output /output