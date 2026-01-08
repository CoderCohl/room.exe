#!/usr/bin/env bash
set -euo pipefail
mkdir -p dist
cargo build --release
cp target/release/room.exe dist/room.exe
