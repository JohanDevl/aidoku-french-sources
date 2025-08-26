#!/usr/bin/env python3

import json
import os
import glob
import re

def slugify(text):
    """Convert text to a slug-like ID"""
    # Convert to lowercase and replace spaces/special chars with hyphens
    slug = re.sub(r'[^\w\s-]', '', text).strip().lower()
    slug = re.sub(r'[\s_-]+', '-', slug)
    return slug

def fix_listing_ids(file_path):
    """Add missing IDs to listings in source.json"""
    print(f"Processing: {file_path}")
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            data = json.load(f)
        
        # Check if there are listings
        if 'listings' in data and isinstance(data['listings'], list):
            modified = False
            for listing in data['listings']:
                if 'id' not in listing and 'name' in listing:
                    # Generate ID from name
                    listing['id'] = slugify(listing['name'])
                    modified = True
                    print(f"  Added ID '{listing['id']}' for listing '{listing['name']}'")
            
            if modified:
                # Write back with proper formatting
                with open(file_path, 'w', encoding='utf-8') as f:
                    json.dump(data, f, indent=2, ensure_ascii=False)
                print(f"  Updated: {file_path}")
            else:
                print(f"  No changes needed: {file_path}")
        else:
            print(f"  No listings found: {file_path}")
    
    except Exception as e:
        print(f"  Error processing {file_path}: {e}")

def main():
    """Find and fix all source.json files"""
    base_dir = "/Users/johan/Developer/GitStore/aidoku-french-sources/src"
    
    # Find all source.json files (excluding target directories)
    pattern = os.path.join(base_dir, "**/source.json")
    files = [f for f in glob.glob(pattern, recursive=True) if 'target' not in f]
    
    print(f"Found {len(files)} source.json files")
    
    for file_path in files:
        fix_listing_ids(file_path)
    
    print("Completed fixing listing IDs!")

if __name__ == "__main__":
    main()