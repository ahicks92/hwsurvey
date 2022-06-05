#!/usr/bin/env bash
set -ex
cd /backend/server
refinery migrate -e DATABASE_URL  -p ./migrations
cargo run
