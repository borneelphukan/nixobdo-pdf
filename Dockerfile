# syntax=docker/dockerfile:1
#
# nixobdo-pdf — GUI PDF viewer in a container.
#
# The app is a native eframe/egui window, so "opening it in Docker" means
# running it on a virtual framebuffer (Xvfb) and streaming the display to a
# browser through noVNC. This image is built for linux/amd64 only, because the
# PDFium binary the app loads at runtime is x64.
#
#   docker build -t nixobdo-pdf .
#   docker run --rm -p 6080:6080 nixobdo-pdf
#   # then open http://localhost:6080/vnc.html?autoconnect=true&resize=scale
#
# Extra arguments are forwarded to the binary, e.g. a PDF to open on start:
#   docker run --rm -p 6080:6080 -v "$PWD:/docs:ro" nixobdo-pdf /docs/report.pdf

###############################################################################
# Stage 1 — build the release binary (linux-x86_64; PDFium is x64)
###############################################################################
FROM --platform=linux/amd64 rust:1.99.1-bookworm AS builder

# Build dependencies mirror .github/workflows/ci.yml (the `check` job), plus
# tooling needed to fetch PDFium. None of it is linked statically: PDFium is a
# runtime dependency the app loads dynamically, so it is only needed in the
# runtime stage.
RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config \
        libgtk-3-dev \
        libx11-dev libxcb1-dev libxcb-composite0-dev libxcb-cursor-dev \
        libxcb-image0-dev libxcb-icccm4-dev libxcb-keysyms1-dev \
        libxcb-randr0-dev libxcb-render0-dev libxcb-render-util0-dev \
        libxcb-shape0-dev libxcb-xfixes0-dev libxcb-xinerama0-dev \
        libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev \
        ca-certificates curl xz-utils git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the full source. `assets/` is required because the splash/about screens
# pull logos in with include_bytes! at compile time. (`.dockerignore` trims the
# build context to what the crate actually needs.)
COPY . .

# Fetch PDFium and drop it in the repo root so it sits beside the binary. This
# matches the release workflow. The file is only copied into the runtime stage;
# the build does not link against it.
RUN mkdir -p /tmp/pdfium \
    && curl -fsSL \
        https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-x64.tgz \
        | tar -xz -C /tmp/pdfium \
    && find /tmp/pdfium -name libpdfium.so -exec cp {} /app/libpdfium.so \; \
    && test -s /app/libpdfium.so

# Build against a locked dependency graph, reusing cargo + target caches across
# rebuilds. Artifacts from a cache mount do not persist in the layer, so the
# finished binary and libpdfium.so are staged under /usr/local/staging for the
# runtime stage to copy out of.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked \
    && mkdir -p /usr/local/staging \
    && cp target/release/nixobdo-pdf /usr/local/staging/nixobdo-pdf \
    && cp libpdfium.so /usr/local/staging/libpdfium.so

###############################################################################
# Stage 2 — runtime: Xvfb + the app + x11vnc + noVNC
###############################################################################
FROM --platform=linux/amd64 debian:bookworm-slim AS runtime

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
        # Virtual X server + software OpenGL (llvmpipe) — no GPU in a container.
        xvfb xauth libgl1 libegl1 libglx-mesa0 libglib2.0-0 \
        # GTK / XCB runtime libs for eframe's window and rfd's native dialogs.
        libgtk-3-0 libxcb1 libxcb-render0 libxcb-shape0 libxcb-xfixes0 \
        libxkbcommon0 libx11-xcb1 libxi6 libxcursor1 libxinerama1 \
        # Fonts so egui can render text at all.
        fontconfig fonts-dejavu-core \
        # Remote desktop: a VNC server plus the noVNC browser client.
        x11vnc novnc websockify \
    && rm -rf /var/lib/apt/lists/*

# Run as an unprivileged user; PDFium + the app need a writable home for the
# settings.json and the render cache under ~/.config and ~/.cache.
RUN useradd -m -u 1000 nixobdo

# The binary looks up PDFium from its own directory first, so both files are
# placed side by side in /usr/local/bin.
COPY --from=builder /usr/local/staging/nixobdo-pdf /usr/local/bin/nixobdo-pdf
COPY --from=builder /usr/local/staging/libpdfium.so /usr/local/bin/libpdfium.so
COPY --chmod=755 docker/entrypoint.sh /usr/local/bin/nixobdo-entrypoint.sh
RUN chmod 755 /usr/local/bin/nixobdo-pdf

USER nixobdo
ENV HOME=/home/nixobdo \
    DISPLAY=:99 \
    RESOLUTION=1600x1000x24 \
    VNC_PORT=5900 \
    NOVNC_PORT=6080

# Only the noVNC browser port is published. x11vnc is bound to loopback inside
# the container and is reachable solely through websockify, so the browser is
# the intended and only way in.
EXPOSE 6080

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD bash -c "exec 3<>/dev/tcp/127.0.0.1/${NOVNC_PORT}" 2>/dev/null || exit 1

ENTRYPOINT ["/usr/local/bin/nixobdo-entrypoint.sh"]
