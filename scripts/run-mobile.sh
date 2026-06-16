#!/usr/bin/env bash
# Run the Flutter mobile app on a simulator/emulator or connected device.
# Fetches Dart deps, prints the server-URL reminder + current join code, then
# hands off to `flutter run` (all arguments are passed through).
#
# Usage: scripts/run-mobile.sh [flutter run args...]
#        scripts/run-mobile.sh -d "iPhone 15"     # target a specific device
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if ! command -v flutter >/dev/null 2>&1; then
  echo "error: flutter not found — install it: https://docs.flutter.dev/get-started/install" >&2
  exit 1
fi

cat <<'EOF'
Server URL to enter in the app's Server screen:
  iOS Simulator     http://localhost:8080
  Android Emulator  http://10.0.2.2:8080      (the app default)
  Physical device   http://<host-LAN-IP>:8080 (e.g. `ipconfig getifaddr en0`)
EOF

db="$ROOT/server/database.db"
if [[ -f "$db" ]] && command -v sqlite3 >/dev/null 2>&1; then
  code="$(sqlite3 "$db" 'SELECT join_code FROM game_server' 2>/dev/null || true)"
  [[ -n "$code" ]] && echo "Join code (for Register): $code"
fi
echo

cd "$ROOT/mobile"
flutter pub get
exec flutter run "$@"
