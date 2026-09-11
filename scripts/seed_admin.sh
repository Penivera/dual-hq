#!/usr/bin/env bash
set -e

# Change directory to the internship-api project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${PROJECT_ROOT}"

echo "==> Running administrator pre-seed script..."
cargo run --bin seed -- "$@"
