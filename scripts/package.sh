#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo build --release --locked
mkdir -p dist/stamina-linux-x86_64
cp target/release/stamina dist/stamina-linux-x86_64/
cp scripts/install.sh README.md dist/stamina-linux-x86_64/
cp assets/stamina.svg dist/stamina-linux-x86_64/
tar -czf dist/stamina-linux-x86_64.tar.gz -C dist stamina-linux-x86_64
