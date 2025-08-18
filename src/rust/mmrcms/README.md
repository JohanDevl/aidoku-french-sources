# MMRCMS Template Sources

Template-based Aidoku sources for MMRCMS (Manga Management & Reader CMS) sites.

## Overview

- **Template Type**: MMRCMS (Manga Management System)
- **Language**: French
- **Architecture**: Macro-based configuration with MMRCMSSource
- **Sources**: 1 total (currently offline)

## Sources

### MangaScan (fr.mangascan)
- **Site**: mangascan-fr.com
- **Status**: ❌ Offline (Website temporarily down)
- **Version**: 1
- **NSFW**: Yes (Adult content)
- **Note**: Maintained for when site returns online

## Template Features

### Core Capabilities
- ✅ MMRCMS framework support
- ✅ Macro-based configuration system
- ✅ Extensive genre/category filtering
- ✅ Search engine with fallback
- ✅ CloudFlare email decoding
- ✅ Manga page caching for performance
- ✅ Self-search fallback mechanism

### Unique Architecture
MMRCMS uses a macro-based configuration system instead of traditional structs:

```rust
#![no_std]
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
    MMRCMSSource {
        base_url: "http://mangascan-fr.com",
        lang: "fr",
        category_mapper: |idx| {
            String::from(match idx {
                1 => "1",   // Action
                2 => "2",   // Adventure  
                3 => "3",   // Comédie
                7 => "7",   // Fantasy
                19 => "19", // Romance
                // ... extensive mapping
                _ => "",
            })
        },
        ..Default::default()
    }
}
```

## Build Commands

```bash
# Build specific source
cd src/rust/mmrcms
./build.sh mangascan

# Build all MMRCMS sources
./build.sh -a
```

## Architecture

```
mmrcms/
├── template/
│   ├── src/
│   │   ├── lib.rs           # Core template with macros
│   │   ├── template.rs      # MMRCMSSource definition  
│   │   └── helper.rs        # Utility functions
│   └── Cargo.toml          # Template dependencies
├── sources/
│   ├── mangascan/          # Site configuration
│   │   ├── src/lib.rs      # mmrcms! macro usage
│   │   ├── res/            # Metadata and filters
│   │   └── Cargo.toml      # Source dependencies
│   └── ...
├── res/
│   ├── Icon.png            # Default template icon
│   └── filters.json        # Shared filter definitions
├── build.sh               # Build script  
└── CLAUDE.md              # Detailed documentation
```

## Configuration System

### MMRCMSSource Fields
- `base_url`: Site base URL (HTTP/HTTPS)
- `lang`: Language code ("fr")
- `manga_path`: Path to manga pages (default: "manga")
- `category_mapper`: Closure mapping filter indices to site category IDs

### Category Mapping System
The `category_mapper` is crucial for proper filtering:

```rust
category_mapper: |idx| {
    String::from(match idx {
        1 => "1",   // Action
        2 => "2",   // Adventure
        3 => "3",   // Comédie
        4 => "4",   // Drame
        5 => "5",   // Ecchi
        6 => "6",   // Erotique
        7 => "7",   // Fantasy
        8 => "8",   // Harem
        9 => "9",   // Historique
        10 => "10", // Horreur
        11 => "11", // Josei
        12 => "12", // Lemon
        13 => "13", // Lime
        14 => "14", // Mature
        15 => "15", // Mecha
        16 => "16", // Mystère
        17 => "17", // Psychologique
        18 => "18", // Reverse Harem
        19 => "19", // Romance
        20 => "20", // School Life
        21 => "21", // Science Fiction
        22 => "22", // Seinen
        23 => "23", // Shojo
        24 => "24", // Shoujo Ai
        25 => "25", // Shounen
        26 => "26", // Shounen Ai
        27 => "27", // Slice of Life
        28 => "28", // Sports
        29 => "29", // Supernatural
        30 => "30", // Tragedy
        31 => "31", // Yaoi
        32 => "32", // Yuri
        _ => "",
    })
}
```

## Template Features

### Advanced Functionality
- **Search Fallback**: Automatically falls back to self-search if primary search fails
- **Email Decoding**: Handles CloudFlare email obfuscation automatically
- **Caching System**: Caches manga pages to reduce server load
- **Error Recovery**: Robust error handling for offline sites

### NSFW Support
- Built-in support for adult content sites
- Proper NSFW flagging in source metadata
- Adult content filtering capabilities

## Adding New Source

1. **Create source directory**:
   ```bash
   mkdir sources/newsitename
   cp -r template/* sources/newsitename/
   ```

2. **Configure `src/lib.rs`** with macro:
   ```rust
   #![no_std]
   use mmrcms_template::{mmrcms, template::MMRCMSSource};
   
   mmrcms! {
       MMRCMSSource {
           base_url: "https://new-site.com",
           lang: "fr",
           category_mapper: |idx| {
               // Map categories to site-specific IDs
               String::from(match idx {
                   1 => "action",
                   2 => "adventure", 
                   // ... etc
                   _ => "",
               })
           },
           ..Default::default()
       }
   }
   ```

3. **Map categories**: Research site's category system and update mapper
4. **Update metadata**: Edit `res/source.json` with site info
5. **Configure filters**: Update `res/filters.json` with available categories
6. **Set NSFW flag**: Add `"nsfw": 1` in source.json if needed
7. **Add icon**: Place site icon in `res/Icon.png`
8. **Test build**: `./build.sh newsitename`

## Maintenance

### Current Status
- **MangaScan Offline**: Primary source currently down
- **Template Maintained**: Ready for new sources or site revival
- **NSFW Compliance**: Proper adult content handling

### Common Issues
- **Site downtime**: MMRCMS sites may go offline frequently
- **Category mapping**: Ensure IDs match target site's system
- **Protocol changes**: Sites may switch HTTP/HTTPS
- **Adult content**: Proper NSFW flagging required

### Troubleshooting
- **Empty results**: Check category mapper accuracy
- **Search failures**: Verify base_url accessibility
- **Missing images**: Check site image hosting
- **SSL errors**: Verify HTTPS support

## Technical Benefits

### Macro System
- **Compile-time generation**: Better performance than runtime configuration
- **Type safety**: Compile-time validation of configuration
- **Code reuse**: Single template supports multiple sites

### Robust Features
- **Fallback mechanisms**: Multiple search strategies
- **Error handling**: Graceful degradation for offline sites
- **Caching**: Improved performance and reduced server load

This template provides a solid foundation for MMRCMS-based French manga sites with extensive genre support and robust error handling, ready for new sources when sites become available.