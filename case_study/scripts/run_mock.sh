#!/bin/bash

echo "=== Running Mock Purchasing Flow ==="
echo ""

cargo run --manifest-path "$(dirname "$0")/../Cargo.toml" --bin purchasing_flow_mock

echo ""
