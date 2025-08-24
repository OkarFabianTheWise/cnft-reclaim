#!/bin/bash
set -e

# Run all Rust unit and integration tests
cargo test --all -- --nocapture
