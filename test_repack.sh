#!/bin/bash

# Test repackaging script
PACKAGE="/Users/johan/Developer/GitStore/aidoku-french-sources/src/rust/fr.lelscanfr/package.aix"
WORK_DIR="/tmp/test_repack"

echo "Testing package repackaging..."

# Clean work directory
rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR"

# Extract package
cd "$WORK_DIR"
unzip -q "$PACKAGE"

echo "Original package structure:"
find . -type f

# Check if Payload exists
if [ -d "Payload" ]; then
    echo "Moving files from Payload/ to root..."
    mv Payload/* .
    rmdir Payload
    echo "New structure:"
    find . -type f
    
    # Repackage
    zip -q -r test_package.aix *
    echo "Repackaged successfully"
    
    # Test with aidoku build
    echo "Testing with aidoku build..."
    cd /Users/johan/Developer/GitStore/aidoku-french-sources
    aidoku build "$WORK_DIR/test_package.aix" -o test_output --name "Test Repack"
    
    echo "Contents of test_output:"
    ls -la test_output/ || echo "No output directory created"
    
    if [ -f "test_output/index.json" ]; then
        echo "index.json contents:"
        cat test_output/index.json
    fi
else
    echo "No Payload directory found"
fi

# Cleanup
rm -rf "$WORK_DIR" test_output/