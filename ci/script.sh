#!/bin/bash

# Run clippy on nightly builds
if [ "$TRAVIS_RUST_VERSION" == "nightly" ]; then
    cargo clippy --all-features --all-targets
fi

# Run test in release mode if a tag is present, to produce an optimized binary
if [ -n "$TRAVIS_TAG" ]; then
    cargo test --release
else
    cargo test
fi

# Test the font subcrate
cargo test -p font
