#!/bin/sh

# Exits immediately if a command exits with a non-zero status
set -e

# PostgreSQL container information
DB_CONTAINER_NAME="test_postgres_1"
DB_NAME="test_db"
DB_USER="Glossy"
DB_PASSWORD="yellau"

# Function to delete entries and measure time taken
delete_entries() {
    docker exec -i "$DB_CONTAINER_NAME" psql -U "$DB_USER" -d "$DB_NAME" -c "DELETE FROM $1;"
}

# Delete entries from different tables and measure time
delete_entries "ships"
delete_entries "empires"
delete_entries "locations"
delete_entries "users"
