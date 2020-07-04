#!/bin/bash

# Run clippy checks
if [ "$CLIPPY" == "true" ]; then
    cargo clippy --all-targets
    exit
fi

# Run test in release mode if a tag is present, to produce an optimized binary
if [ -n "$TRAVIS_TAG" ]; then
    # Build separately so we generate an 'alacritty' binary without -HASH appended
    cargo build --release
    cargo test --release
else
    cargo test
fi

# Output binary name
name="Alacritty-${TRAVIS_TAG}"

# Everything in this directory will be offered as download for the release
mkdir "./target/deploy"

rm -rf "./target/release" \
    && make dmg \
    && mv "./target/release/osx/Alacritty.dmg" "./target/deploy/${name}.dmg"

ls -lah target/release/osx/Alacritty.app/Contents/Resources/*
