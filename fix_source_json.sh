#!/bin/bash

# Script to fix all source.json files to use the correct format

echo "Fixing source.json files..."

# Find all source.json files (excluding target directories)
find src -name "source.json" | grep -v target | while read -r file; do
    echo "Processing: $file"
    
    # Create backup
    cp "$file" "$file.backup"
    
    # Apply fixes using sed
    sed -i.tmp \
        -e 's/"lang": "fr",/"contentRating": 1,\n    "languages": ["fr"],/g' \
        -e 's/\t\t"lang": "fr",/\t\t"contentRating": 1,\n\t\t"languages": ["fr"],/g' \
        -e 's/"nsfw": 0//' \
        -e 's/,\s*$//' \
        "$file"
    
    # Remove temporary file
    rm -f "$file.tmp"
    
    echo "Fixed: $file"
done

echo "All source.json files have been updated!"
echo "Backups created with .backup extension"