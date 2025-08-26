#!/bin/bash

# Test the exact same command as used in the workflow

echo "=== Testing exact workflow command locally ==="

# The exact files from the workflow log
AIX_FILES="./src/rust/fr.animesama/package.aix ./src/rust/fr.legacyscans/package.aix ./src/rust/fr.lelscanfr/package.aix ./src/rust/fr.phenixscans/package.aix ./src/rust/fr.poseidonscans/package.aix ./src/rust/madara/astralmanga.aix ./src/rust/madara/lelmanga.aix ./src/rust/madara/mangascantrad.aix ./src/rust/madara/mangasorigines.aix ./src/rust/madara/reaperscansfr.aix ./src/rust/mangastream/sushiscan.aix ./src/rust/mangastream/sushiscans.aix ./src/rust/mmrcms/mangascan.aix"

SOURCE_NAME="JohanDevl's French Sources (Development)"

echo "AIX_FILES: $AIX_FILES"
echo "SOURCE_NAME: $SOURCE_NAME"

# First, let's check which files actually exist
echo "=== Checking which files exist ==="
for file in $AIX_FILES; do
    if [ -f "$file" ]; then
        echo "✓ EXISTS: $file"
    else
        echo "✗ MISSING: $file"
    fi
done

# Now try the exact command
echo "=== Running exact workflow command ==="
echo "Command: aidoku build $AIX_FILES -o workflow_test --name \"$SOURCE_NAME\""

# First create the packages that might be missing
echo "=== Creating missing packages ==="
cd src/rust/fr.animesama && RUSTUP_TOOLCHAIN=nightly aidoku package && echo "✓ animesama package created" || echo "✗ animesama failed"
cd ../fr.legacyscans && RUSTUP_TOOLCHAIN=nightly aidoku package && echo "✓ legacyscans package created" || echo "✗ legacyscans failed"
cd ../fr.lelscanfr && RUSTUP_TOOLCHAIN=nightly aidoku package && echo "✓ lelscanfr package created" || echo "✗ lelscanfr failed"
cd ../fr.poseidonscans && RUSTUP_TOOLCHAIN=nightly aidoku package && echo "✓ poseidonscans package created" || echo "✗ poseidonscans failed"
cd ../../..

# Check what packages we have now
echo "=== Available packages after creation ==="
find src -name "*.aix" -type f | sort

# Try with just existing packages
EXISTING_FILES=""
for file in $AIX_FILES; do
    if [ -f "$file" ]; then
        EXISTING_FILES="$EXISTING_FILES $file"
    fi
done

echo "EXISTING_FILES: $EXISTING_FILES"

if [ -n "$EXISTING_FILES" ]; then
    echo "=== Testing with existing packages ==="
    aidoku build $EXISTING_FILES -o workflow_test --name "$SOURCE_NAME"
    
    echo "=== Results ==="
    echo "Exit code: $?"
    ls -la workflow_test/
    echo "Index.json:"
    cat workflow_test/index.json
    echo "Sources count:"
    cat workflow_test/index.json | python3 -c "import json,sys; data=json.load(sys.stdin); print(len(data['sources']))"
else
    echo "No existing packages found!"
fi