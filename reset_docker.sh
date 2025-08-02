#!/bin/sh

# Exits immediately if a command exits with a non-zero status
set -e

# Run docker-compose down for wiping databases
docker-compose -f db/dev/docker-compose.yml down
docker-compose -f db/test/docker-compose.yml down