#!/bin/sh
#
# Add the nixobdo-pdf apt repository and install the app.
#
#   curl -fsSL https://borneelphukan.github.io/nixobdo-pdf/install.sh | sudo bash
#
# Options:
#   --print, -n   print the commands instead of running them
#   --remove      remove the repository (the installed package stays untouched)
#   --yes, -y     assume yes for apt-get
#   --help, -h    show this help
#
# Environment:
#   NIXOBDO_PDF_REPO_URL   override the repository base URL
#
# This script is meant to be piped in through sudo, so it never consumes stdin,
# and it re-executes itself with sudo when started as a normal user.

set -eu

# The parser below consumes "$@" with shift, but the sudo re-exec further down
# must forward exactly the options the user passed. Every accepted option is a
# whitespace-free flag, so a plain string is safe to replay.
ORIGINAL_ARGS="$*"

BASE_URL="${NIXOBDO_PDF_REPO_URL:-https://borneelphukan.github.io/nixobdo-pdf}"
SUITE="stable"
COMPONENT="main"
ARCH="$(dpkg --print-architecture 2>/dev/null || uname -m)"
KEYRING="/usr/share/keyrings/nixobdo-pdf.gpg"
SOURCES="/etc/apt/sources.list.d/nixobdo-pdf.sources"
KEY_URL="$BASE_URL/nixobdo-pdf-archive-keyring.gpg"
PACKAGE="nixobdo-pdf"
ASSUME_YES=""
MODE="install"
DRY_RUN=0

usage() {
    cat <<'EOF'
Usage: install.sh [--print] [--remove] [--yes] [--help]

  --print, -n   print the commands instead of running them
  --remove      remove the repository (the installed package stays untouched)
  --yes, -y     assume yes for apt-get
  --help, -h    show this help
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --print|-n) DRY_RUN=1 ;;
        --remove) MODE="remove" ;;
        --yes|-y) ASSUME_YES="-y" ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            printf 'unknown option: %s\n\n' "$1" >&2
            usage >&2
            exit 2
            ;;
    esac
    shift
done

say() { printf '%s\n' "$*"; }
note() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
run() {
    if [ "$DRY_RUN" -eq 1 ]; then
        printf '+ %s\n' "$*"
    else
        "$@"
    fi
}

fetch() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$1" -o "$2"
    else
        wget -qO "$2" "$1"
    fi
}

# The public key is published next to the repository. When it is missing, the
# repository was published without a signing key (forks and pull requests) and
# apt has to be told to trust it explicitly.
SIGNED=1
if [ "$DRY_RUN" -eq 1 ]; then
    say "(dry run: not contacting $KEY_URL)"
elif ! fetch "$KEY_URL" /dev/null 2>/dev/null; then
    SIGNED=0
fi

if [ "$SIGNED" -eq 1 ]; then
    TRUSTED_LINE="Signed-By: $KEYRING"
else
    KEYRING=""
    TRUSTED_LINE="Trusted: yes"
    note "no signing key published at $KEY_URL, using Trusted: yes"
fi

sources_body() {
    printf 'Types: deb\n'
    printf 'URIs: %s/apt/\n' "$BASE_URL"
    printf 'Suites: %s\n' "$SUITE"
    printf 'Components: %s\n' "$COMPONENT"
    printf 'Architectures: %s\n' "$ARCH"
    printf '%s\n' "$TRUSTED_LINE"
}

if [ "$MODE" = "remove" ]; then
    run rm -f "$SOURCES"
    if [ -n "$KEYRING" ]; then
        run rm -f "$KEYRING"
    fi
    say "repository removed. Uninstall the app with: sudo apt remove $PACKAGE"
    exit 0
fi

if [ "$DRY_RUN" -eq 0 ] && [ "$(id -u)" -ne 0 ]; then
    if command -v sudo >/dev/null 2>&1; then
        # shellcheck disable=SC2086  # replaying simple flags, see ORIGINAL_ARGS
        exec sudo -- "$0" $ORIGINAL_ARGS
    fi
    say "error: this script needs root privileges. Try: sudo $0 $*" >&2
    exit 1
fi

if [ -n "$KEYRING" ]; then
    note "installing the repository signing key into $KEYRING"
    run fetch "$KEY_URL" "$KEYRING.tmp"
    if [ "$DRY_RUN" -eq 0 ]; then
        mv -f "$KEYRING.tmp" "$KEYRING"
        chmod 0644 "$KEYRING"
    fi
fi

note "writing $SOURCES"
if [ "$DRY_RUN" -eq 1 ]; then
    printf '+ cat > %s <<EOF\n' "$SOURCES"
    sources_body
    printf 'EOF\n'
else
    sources_body > "$SOURCES.tmp"
    chmod 0644 "$SOURCES.tmp"
    mv -f "$SOURCES.tmp" "$SOURCES"
fi

note "refreshing the package index"
run apt-get update

note "installing $PACKAGE"
run apt-get install $ASSUME_YES "$PACKAGE"

[ "$DRY_RUN" -eq 0 ] && say "nixobdo-pdf is installed. Start it with: $PACKAGE"
exit 0
