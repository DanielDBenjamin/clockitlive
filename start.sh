#!/bin/bash

# Detect if we're running locally or in Railway
if [ -d "/app" ]; then
    # Railway environment
    DATA_DIR="/app/data"
    BINARY="./clock-it"
    SITE_ROOT="/app/target/site"
else
    # Local environment
    DATA_DIR="./data"
    BINARY="./target/release/clock-it"
    SITE_ROOT="./target/site"
fi

# Set up the database directory
mkdir -p "$DATA_DIR"

# Verify site directory exists
if [ ! -d "$SITE_ROOT" ]; then
    echo "ERROR: Site directory not found at $SITE_ROOT"
    exit 1
fi

# List contents for debugging
echo "Contents of $SITE_ROOT:"
ls -la "$SITE_ROOT"

if [ -d "$SITE_ROOT/pkg" ]; then
    echo "Contents of $SITE_ROOT/pkg:"
    ls -la "$SITE_ROOT/pkg"
fi

# Check if database exists, if not create it
if [ ! -f "$DATA_DIR/clock_it.db" ]; then
    echo "Database not found, creating new database..."
    touch "$DATA_DIR/clock_it.db"
    echo "Database created at $DATA_DIR/clock_it.db"
fi

# Set environment variables for Railway
export DATABASE_URL="sqlite:$DATA_DIR/clock_it.db"
export LEPTOS_SITE_ROOT="$SITE_ROOT"
export LEPTOS_SITE_PKG_DIR="pkg"

echo "Starting application..."
echo "DATABASE_URL: $DATABASE_URL"
echo "LEPTOS_SITE_ROOT: $LEPTOS_SITE_ROOT"

# Start the application
exec "$BINARY"