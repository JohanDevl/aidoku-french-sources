#!/bin/bash

# Script to repair JSON formatting after sed modifications

echo "Repairing JSON formatting..."

# Function to repair a single JSON file
repair_json_file() {
    local file="$1"
    echo "Repairing: $file"
    
    # Restore backup if it exists
    if [ -f "$file.backup" ]; then
        cp "$file.backup" "$file"
        echo "Restored from backup: $file"
        
        # Apply correct transformations
        python3 -c "
import json
import sys

# Read the file
with open('$file', 'r') as f:
    data = json.load(f)

# Apply transformations
if 'info' in data:
    info = data['info']
    
    # Remove old fields
    if 'lang' in info:
        del info['lang']
    if 'nsfw' in info:
        del info['nsfw']
    
    # Add new fields
    info['contentRating'] = 1
    info['languages'] = ['fr']

# Write back
with open('$file', 'w') as f:
    json.dump(data, f, indent=2, ensure_ascii=False)

print('Repaired JSON successfully')
"
        
        if [ $? -eq 0 ]; then
            echo "Successfully repaired: $file"
            rm -f "$file.backup"
        else
            echo "Failed to repair: $file"
        fi
    else
        echo "No backup found for: $file"
    fi
}

# Find and repair all source.json files
find src -name "source.json" | grep -v target | while read -r file; do
    repair_json_file "$file"
done

echo "JSON repair completed!"