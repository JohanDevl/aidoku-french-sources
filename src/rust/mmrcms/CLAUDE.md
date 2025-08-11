# CLAUDE.md - MMRCMS Template

This file provides guidance for working with MMRCMS (Manga Management System) template sources in this repository.

## MMRCMS Template Architecture

The MMRCMS template supports manga management system sites with a macro-based configuration system.

### Current Sources
- **mangascan**: Manga Scan (mangascan-fr.com) - Currently offline

## Build Commands

```bash
# Build specific source
./build.sh mangascan

# Build all MMRCMS sources  
./build.sh -a
```

## Source Configuration

MMRCMS sources use a macro-based configuration with `MMRCMSSource` struct:

```rust
#![no_std]
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
    MMRCMSSource {
        base_url: "http://mangascan-fr.com",
        lang: "fr",
        category_mapper: |idx| {
            String::from(match idx {
                1 => "1", // Action
                2 => "2", // Adventure
                3 => "3", // Comédie
                // ... other categories
                _ => "",
            })
        },
        ..Default::default()
    }
}
```

### Key Configuration Fields
- `base_url`: Site base URL
- `lang`: Language code ("fr")
- `category_mapper`: Closure mapping category indices to site-specific IDs
- `manga_path`: Path to manga pages (default: "manga")

### Category Mapping
The `category_mapper` function maps filter indices to site category IDs:
- Action (1) → "1"  
- Adventure (2) → "2"
- Comédie (3) → "3"
- Fantasy (7) → "7"
- Romance (19) → "19"
- Seinen (22) → "22"
- Shounen (25) → "25"
- And many more...

## Adding New MMRCMS Source

1. **Create source directory**: `sources/newsitename/`
2. **Copy template structure**:
   ```bash
   cp -r template/ sources/newsitename/
   ```
3. **Configure `src/lib.rs`**: Update `MMRCMSSource` configuration
4. **Map categories**: Update `category_mapper` with site-specific category IDs
5. **Update `res/source.json`**: Set source metadata
6. **Configure `res/filters.json`**: Define available category filters
7. **Add icon**: Place `Icon.png` in `res/`
8. **Test build**: `./build.sh newsitename`

## Common Issues

- **Category mapping**: Ensure category IDs match the target site's system
- **Protocol**: Check if site uses HTTP vs HTTPS
- **NSFW content**: Set `nsfw: 1` in `source.json` if site contains adult content
- **Site downtime**: MMRCMS sites may go offline frequently
- **Caching**: Template uses manga page caching for performance

## Template Features

- **Search engine fallback**: Automatically falls back to self-search if primary search fails
- **Email decoding**: Handles CloudFlare email obfuscation
- **Manga caching**: Caches manga pages to reduce requests
- **Flexible category system**: Supports extensive genre/category filtering