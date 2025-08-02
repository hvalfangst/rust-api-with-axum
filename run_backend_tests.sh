#!/bin/sh

# Exits immediately if a command exits with a non-zero status
set -e

# Function to get the current timestamp in milliseconds
timestamp_ms() {
  date +%s%3N
}

start_time_db_reset=$(timestamp_ms)

# Wipes our test database
sh backend/db/test/reset.sh

end_time_db_reset=$(timestamp_ms)
elapsed_time_db_reset=$((end_time_db_reset - start_time_db_reset))

start_time_cargo_test=$(timestamp_ms)

# Run our tests with only one thread to mitigate race conditions
cargo test -- --test-threads=1

end_time_cargo_test=$(timestamp_ms)
elapsed_time_cargo_test=$((end_time_cargo_test - start_time_cargo_test))
elapsed_time_total=$((elapsed_time_db_reset + elapsed_time_cargo_test))

echo "Script finished in ${elapsed_time_total} ms"
echo "DB reset finished in ${elapsed_time_db_reset} ms"
echo "Cargo test finished in ${elapsed_time_cargo_test} ms"