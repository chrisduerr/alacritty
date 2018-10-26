#!/bin/bash

# Install choco for windows builds
if [ "$TRAVIS_OS_NAME" == "windows" ]; then
    choco install llvm --norestart --nosilent || exit 1
fi

# Add clippy for linting with nightly builds
if [ "$TRAVIS_RUST_VERSION" == "nightly" ]; then
    rustup component add clippy-preview
fi
