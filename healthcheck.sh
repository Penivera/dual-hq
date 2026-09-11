#!/bin/bash
set -e

PORT="${SERVER_PORT:-8010}"
curl -f "http://127.0.0.1:${PORT}/api/health" || exit 1
