#!/bin/bash

# Debug aidoku build with detailed output

echo "=== Creating test package ==="
WORK_DIR="/tmp/debug_aidoku"
rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR"
cd "$WORK_DIR"

# Extract original package
unzip -q /Users/johan/Developer/GitStore/aidoku-french-sources/src/rust/fr.lelscanfr/package.aix

echo "Original structure:"
ls -la

if [ -d "Payload" ]; then
    echo "Moving files from Payload to root..."
    mv Payload/* .
    rmdir Payload
fi

echo "New structure:"
ls -la

echo "source.json contents:"
cat source.json

echo "=== Creating new package ==="
zip -r new_package.aix Icon.png source.json main.wasm filters.json

echo "=== Testing with aidoku build ==="
cd /Users/johan/Developer/GitStore/aidoku-french-sources

echo "Running: aidoku build $WORK_DIR/new_package.aix -o debug_test --name 'Debug Test'"
aidoku build "$WORK_DIR/new_package.aix" -o debug_test --name "Debug Test" 2>&1

echo "Exit code: $?"

echo "=== Output directory contents ==="
ls -la debug_test/

if [ -f "debug_test/index.json" ]; then
    echo "index.json contents:"
    cat debug_test/index.json
else
    echo "No index.json created"
fi

# Cleanup
rm -rf debug_test "$WORK_DIR"