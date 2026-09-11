#!/bin/sh
set -e

# Generate JWT_SECRET if not provided
if [ -z "$JWT_SECRET" ]; then
    export JWT_SECRET="$(head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n')"
fi

exec /usr/local/bin/internship-api "$@"
