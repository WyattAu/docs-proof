#!/usr/bin/env bash
# Build the REQ-P4 browser checker cdylib and place it at the path in $1
# (default assets/docs-proof.wasm). Plain script, not a template: this
# file is executed from the source dir (fleet rule). Respects
# CARGO_TARGET_DIR so local builds land in the shared dedup target.
set -euo pipefail

out="${1:-assets/docs-proof.wasm}"
target_dir="${CARGO_TARGET_DIR:-target}"

rustup target add wasm32-unknown-unknown
# A lib crate emits no .wasm from plain `build`; the cdylib crate-type
# produces the browser artifact.
cargo rustc --release --target wasm32-unknown-unknown --crate-type cdylib

src="$target_dir/wasm32-unknown-unknown/release/docs_proof.wasm"
mkdir -p "$(dirname "$out")"
cp "$src" "$out"
printf '%s: %s bytes\n' "$out" "$(wc -c < "$out")"
