#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo build --release --locked
mkdir -p dist/capee-linux-x86_64
cp target/release/capee dist/capee-linux-x86_64/
cp scripts/install.sh README.md dist/capee-linux-x86_64/
cp assets/capee.svg dist/capee-linux-x86_64/
tar -czf dist/capee-linux-x86_64.tar.gz -C dist capee-linux-x86_64
