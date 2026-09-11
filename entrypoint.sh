#!/bin/bash
set -e

SERVICE_TYPE="${SERVICE_TYPE:-web}"

echo "Starting container with SERVICE_TYPE=${SERVICE_TYPE}"

case "$SERVICE_TYPE" in
    "web")
        echo "--> Spawning Internship Application API Server..."
        exec /usr/local/bin/internship-api
        ;;
    "seed")
        echo "--> Spawning Administrator Seeding Utility..."
        exec /usr/local/bin/seed "$@"
        ;;
    *)
        exec "$@"
        ;;
esac
