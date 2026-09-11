#!/bin/bash
set -e

SERVICE_TYPE="${SERVICE_TYPE:-web}"

# Auto-generate or load persistent JWT_SECRET if not provided or set to default placeholder
if [ -z "$JWT_SECRET" ] || [ "$JWT_SECRET" = "super-secret-jwt-key-replace-in-production" ]; then
    SECRET_FILE="/app/logs/.jwt_secret"
    if [ -f "$SECRET_FILE" ] && [ -s "$SECRET_FILE" ]; then
        JWT_SECRET="$(cat "$SECRET_FILE" | tr -d ' \r\n')"
        echo "--> Loaded existing JWT secret from ${SECRET_FILE}"
    else
        if command -v openssl >/dev/null 2>&1; then
            JWT_SECRET=$(openssl rand -hex 32)
        else
            JWT_SECRET=$(head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n')
        fi
        if [ -d "/app/logs" ]; then
            echo "$JWT_SECRET" > "$SECRET_FILE"
            chmod 600 "$SECRET_FILE" 2>/dev/null || true
            echo "--> Generated and persisted secure random JWT secret to ${SECRET_FILE}"
        else
            echo "--> Generated secure random in-memory JWT secret"
        fi
    fi
    export JWT_SECRET
    export APP_JWT_SECRET="$JWT_SECRET"
fi

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
