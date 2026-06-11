#!/usr/bin/env bash
# Regenerate Dart types for the Lorewyld mobile app from the shared
# typeshare-annotated Rust crate.
#
# Prereq: cargo install typeshare-cli
set -euo pipefail

cd "$(dirname "$0")/.."

mkdir -p mobile/lib/types

typeshare shared/types \
  --lang=dart \
  --output-folder=mobile/lib/types

echo "Regenerated Dart types in mobile/lib/types/"
