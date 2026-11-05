#!/bin/sh
# Installs the latest sorter release to ~/.local/bin.
# Usage: curl -fsSL https://raw.githubusercontent.com/abhishek-shivale/sorter/main/install.sh | sh
set -eu

BASE_URL="https://github.com/abhishek-shivale/sorter/releases/latest/download"
INSTALL_DIR="$HOME/.local/bin"

die() {
    echo "sorter install: $1" >&2
    exit 1
}

fetch() {
    curl --proto '=https' --tlsv1.2 -fsSL -o "$1" "$BASE_URL/$1" || die "could not download $BASE_URL/$1"
}

# Everything runs from main, so a truncated download cannot run half a script.
main() {
    [ "$(uname -s)" = "Darwin" ] || die "only macOS is supported"

    case "$(uname -m)" in
        arm64) target="aarch64-apple-darwin" ;;
        x86_64) target="x86_64-apple-darwin" ;;
        *) die "unsupported architecture: $(uname -m)" ;;
    esac
    asset="sorter-$target.tar.gz"

    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    cd "$tmp"

    fetch "$asset"
    fetch "$asset.sha256"
    shasum -a 256 -c "$asset.sha256" >/dev/null 2>&1 || die "checksum mismatch for $asset"
    tar -xzf "$asset"

    mkdir -p "$INSTALL_DIR"
    cp sorter "$INSTALL_DIR/sorter.new"
    chmod 755 "$INSTALL_DIR/sorter.new"
    # Rename, not overwrite: rewriting a signed binary in place makes macOS kill it on launch.
    mv -f "$INSTALL_DIR/sorter.new" "$INSTALL_DIR/sorter"

    echo "Installed $("$INSTALL_DIR/sorter" --version) to $INSTALL_DIR/sorter"
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *) echo "Add it to your PATH: export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
    esac
    echo "If the daemon is already running, load the new version: sorter stop && sorter start"
}

main
