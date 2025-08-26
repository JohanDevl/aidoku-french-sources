#!/bin/bash

# Final test script
WORK_DIR="/tmp/final_test_aidoku"
BASE_DIR="/Users/johan/Developer/GitStore/aidoku-french-sources"

echo "=== Final test of corrected source.json format ==="

# Clean and create work directory
rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR"
cd "$WORK_DIR"

# Extract package
echo "Extracting package..."
unzip -q "$BASE_DIR/src/rust/fr.lelscanfr/package.aix"

# Verify source.json format
echo "Source.json format:"
cat Payload/source.json

# Move files to root and repackage
echo "Repackaging without Payload structure..."
mv Payload/* .
rm -rf Payload
zip -q -r corrected_package.aix *

# Test with aidoku build
echo "Testing with aidoku build..."
cd "$BASE_DIR"
aidoku build "$WORK_DIR/corrected_package.aix" -o final_test_output --name "Corrected Format Test"

echo "Exit code: $?"

echo "Output contents:"
ls -la final_test_output/

echo "Index.json:"
cat final_test_output/index.json

echo "Sources directory:"
ls -la final_test_output/sources/

# Cleanup
rm -rf final_test_output "$WORK_DIR"