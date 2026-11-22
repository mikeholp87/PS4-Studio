#!/usr/bin/env bash
# Regenerates UI test fixtures from the real Rust capability engine + simulator scenarios.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo run -q -p ps4studio-mock-console -- --json > apps/desktop/e2e/fixtures/scenarios.json
echo "wrote apps/desktop/e2e/fixtures/scenarios.json"
