#!/usr/bin/env bash
set -euo pipefail

APP="crabchat"
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
TARGET_DIR="target"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

usage() {
    cat <<EOF
Usage: ./build.sh [command]

Commands:
  dev         Build debug binary (default)
  release     Build optimized release binary
  test        Run all tests
  check       Run cargo check + clippy
  clean       Remove build artifacts
  install     Build release and install to $INSTALL_DIR
  uninstall   Remove installed binary
  run         Build debug and run
  fmt         Format code
  size        Show release binary size

Environment:
  INSTALL_DIR   Installation directory (default: /usr/local/bin)
EOF
}

cmd_dev() {
    echo "==> Building debug binary..."
    cargo build
    echo "==> Built: ${TARGET_DIR}/debug/${APP}"
}

cmd_release() {
    echo "==> Building release binary..."
    cargo build --release
    local size
    size=$(ls -lh "${TARGET_DIR}/release/${APP}" | awk '{print $5}')
    echo "==> Built: ${TARGET_DIR}/release/${APP} (${size})"
}

cmd_test() {
    echo "==> Running tests..."
    cargo test
}

cmd_check() {
    echo "==> Running cargo check..."
    cargo check
    echo "==> Running clippy..."
    cargo clippy -- -D warnings 2>/dev/null || cargo clippy
}

cmd_clean() {
    echo "==> Cleaning build artifacts..."
    cargo clean
    echo "==> Done."
}

cmd_install() {
    cmd_release
    echo "==> Installing to ${INSTALL_DIR}/${APP}..."
    install -d "${INSTALL_DIR}"
    install -m 755 "${TARGET_DIR}/release/${APP}" "${INSTALL_DIR}/${APP}"
    echo "==> Installed ${APP} v${VERSION} to ${INSTALL_DIR}/${APP}"
}

cmd_uninstall() {
    if [ -f "${INSTALL_DIR}/${APP}" ]; then
        echo "==> Removing ${INSTALL_DIR}/${APP}..."
        rm -f "${INSTALL_DIR}/${APP}"
        echo "==> Uninstalled."
    else
        echo "==> ${APP} is not installed at ${INSTALL_DIR}/${APP}"
    fi
}

cmd_run() {
    echo "==> Building and running..."
    cargo run
}

cmd_fmt() {
    echo "==> Formatting code..."
    cargo fmt
    echo "==> Done."
}

cmd_size() {
    if [ ! -f "${TARGET_DIR}/release/${APP}" ]; then
        cmd_release
    fi
    echo ""
    echo "Binary: ${TARGET_DIR}/release/${APP}"
    ls -lh "${TARGET_DIR}/release/${APP}" | awk '{print "Size:  ", $5}'
}

# Main
COMMAND="${1:-dev}"
case "$COMMAND" in
    dev)        cmd_dev ;;
    release)    cmd_release ;;
    test)       cmd_test ;;
    check)      cmd_check ;;
    clean)      cmd_clean ;;
    install)    cmd_install ;;
    uninstall)  cmd_uninstall ;;
    run)        cmd_run ;;
    fmt)        cmd_fmt ;;
    size)       cmd_size ;;
    -h|--help|help) usage ;;
    *)
        echo "Unknown command: $COMMAND"
        usage
        exit 1
        ;;
esac
