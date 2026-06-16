#!/usr/bin/env bash
# Rebuild the web client's WASM bindings to the shared Rust core
# (lorewyld-domain via lorewyld-domain-wasm) and refresh the assets the
# server serves from server/assets/wasm/.
#
# Run after changing lorewyld-domain or lorewyld-domain-wasm.
# Prereqs: cargo install wasm-pack; rustup target add wasm32-unknown-unknown.
# The wasm getrandom backend cfg lives in .cargo/config.toml.
set -euo pipefail

cd "$(dirname "$0")/.."
wasm-pack build shared/domain-wasm \
  --target web \
  --out-dir ../../server/assets/wasm \
  --out-name lorewyld_domain \
  --release

# wasm-pack writes a `.gitignore` (`*`) into the out-dir each run; remove it
# so the served artifacts stay tracked (there is no CI build step).
rm -f server/assets/wasm/.gitignore
echo "Rebuilt web WASM at server/assets/wasm/"
