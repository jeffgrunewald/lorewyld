#!/usr/bin/env bash
# Build and run the Lorewyld server for local development. Listens on the
# address in server/config.toml (0.0.0.0:8080 by default) and serves the JSON
# API, the web UI, and Swagger from one binary. On first start it migrates the
# DB, seeds SRD content, and generates a join code.
#
# Env:
#   LW_RELEASE=1       build/run in release mode (default: debug)
#   LW__API_ADDR=...   override the listen address (e.g. 0.0.0.0:9000)
#   (any LW__* var overrides the matching config.toml key)
#
# Usage: scripts/run-backend.sh [extra args passed to the `server` subcommand]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/server"

profile_flags=()
[[ "${LW_RELEASE:-}" == "1" ]] && profile_flags=(--release)

cat <<EOF
Starting Lorewyld server...
  Web UI:   http://localhost:8080
  Swagger:  http://localhost:8080/swagger-ui
  Admin:    username 'admin' / password 'admin'

Once it is up, read the join code (in another shell) with:
  sqlite3 "$ROOT/server/database.db" 'SELECT join_code FROM game_server'

EOF

exec cargo run ${profile_flags[@]+"${profile_flags[@]}"} -- -c config.toml server "$@"
