#!/usr/bin/env bash
# One-time local dev setup: verify toolchains and fetch dependencies for the
# Rust workspace and the Flutter app. Safe to re-run.
#
# Usage: scripts/setup-dev.sh
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo not found — install Rust (edition 2024): https://rustup.rs" >&2
  exit 1
fi

echo "==> Fetching Rust workspace dependencies"
cargo fetch

echo "==> Building the server (debug)"
cargo build -p lorewyld

if command -v flutter >/dev/null 2>&1; then
  echo "==> Fetching Flutter app dependencies"
  (cd mobile && flutter pub get)
else
  echo "note: flutter not found — skipping mobile setup. Install it to run the app:" >&2
  echo "      https://docs.flutter.dev/get-started/install" >&2
fi

cat <<'EOF'

Setup complete. Next:
  scripts/run-backend.sh        # start the server (http://localhost:8080)
  scripts/run-mobile.sh         # launch the app on a simulator/emulator

See docs/LOCAL_DEVELOPMENT.md for connecting the app to the server.
EOF
