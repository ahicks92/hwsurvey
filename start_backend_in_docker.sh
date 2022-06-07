#!/usr/bin/env bash
set -ex
cd /backend/server
# The database can take a minute to come up.
sleep 5
refinery migrate -e DATABASE_URL  -p ./migrations
cargo run -- --address 0.0.0.0 --port 10000
