#!/bin/bash

echo "=== Rebuilding template packages ==="

# Remove old packages
find src/rust -name "*.aix" -not -path "*/target/*" -delete
echo "Removed old packages"

# Rebuild madara templates
echo "Building Madara templates..."
cd src/rust/madara
./build.sh -a
cd ../../..

# Rebuild mangastream templates  
echo "Building MangaStream templates..."
cd src/rust/mangastream
./build.sh -a
cd ../../..

# Rebuild mmrcms templates
echo "Building MMRCMS templates..."
cd src/rust/mmrcms
./build.sh
cd ../../..

# Rebuild individual sources
echo "Building individual sources..."
cd src/rust/fr.animesama && RUSTUP_TOOLCHAIN=nightly aidoku package && cd ../../..
cd src/rust/fr.legacyscans && RUSTUP_TOOLCHAIN=nightly aidoku package && cd ../../..
cd src/rust/fr.lelscanfr && RUSTUP_TOOLCHAIN=nightly aidoku package && cd ../../..
cd src/rust/fr.phenixscans && RUSTUP_TOOLCHAIN=nightly aidoku package && cd ../../..
cd src/rust/fr.poseidonscans && RUSTUP_TOOLCHAIN=nightly aidoku package && cd ../../..

echo "=== All packages rebuilt ==="
find src/rust -name "*.aix" -not -path "*/target/*" | sort