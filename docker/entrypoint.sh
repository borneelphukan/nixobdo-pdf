#!/usr/bin/env bash
#
# Brings up a headless desktop for nixobdo-pdf:
#   Xvfb (virtual display) -> x11vnc (VNC server) -> websockify/noVNC (browser)
# and then runs the viewer itself in the foreground.
set -euo pipefail

DISPLAY_NUM="${DISPLAY#:}"
XVFB_PID=""

cleanup() {
    [ -n "$XVFB_PID" ] && kill "$XVFB_PID" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

echo "==> Starting Xvfb on ${DISPLAY} at ${RESOLUTION}"
Xvfb "$DISPLAY" -screen 0 "$RESOLUTION" -nolisten tcp >/tmp/xvfb.log 2>&1 &
XVFB_PID=$!

# Wait for the X socket before attaching anything to the display.
for _ in $(seq 1 50); do
    [ -e "/tmp/.X11-unix/X${DISPLAY_NUM}" ] && break
    sleep 0.1
done
if [ ! -e "/tmp/.X11-unix/X${DISPLAY_NUM}" ]; then
    echo "Xvfb failed to start; see /tmp/xvfb.log" >&2
    cat /tmp/xvfb.log >&2 || true
    exit 1
fi

echo "==> Starting x11vnc on :${VNC_PORT} (loopback only)"
# -localhost restricts the VNC server to 127.0.0.1, so it can only be reached
# through websockify/noVNC below. The browser is the sole way in.
if [ -n "${VNC_PASSWORD:-}" ]; then
    x11vnc -display "$DISPLAY" -forever -shared -localhost \
        -rfbport "$VNC_PORT" -passwd "$VNC_PASSWORD" -bg -o /tmp/x11vnc.log
else
    x11vnc -display "$DISPLAY" -forever -shared -localhost \
        -nopw -rfbport "$VNC_PORT" -bg -o /tmp/x11vnc.log
fi

echo "==> Starting noVNC (websockify) on :${NOVNC_PORT}"
websockify --web /usr/share/novnc "$NOVNC_PORT" "localhost:${VNC_PORT}" \
    >/tmp/websockify.log 2>&1 &

cat <<EOF

nixobdo-pdf is launching. Open it in a browser:
  http://localhost:${NOVNC_PORT}/vnc.html?autoconnect=true&resize=scale

The VNC server itself is bound to loopback and is not reachable directly; if
you set VNC_PASSWORD, noVNC will prompt for it after connecting.

EOF

# Run the viewer in the foreground. Any container arguments (for example a PDF
# path) are forwarded to the binary.
exec nixobdo-pdf "$@"
