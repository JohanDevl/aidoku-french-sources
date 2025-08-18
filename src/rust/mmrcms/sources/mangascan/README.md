# MangaScan Source

MMRCMS template-based source for [mangascan-fr.com](https://mangascan-fr.com).

## Overview

- **Site**: mangascan-fr.com
- **Template**: MMRCMS (Manga Management System)
- **Status**: ❌ Offline (Website temporarily down)
- **Language**: French
- **Version**: 1
- **NSFW**: Yes (Adult content)

## Features

### MMRCMS Capabilities (When Active)
- ✅ Extensive genre/category filtering (32 categories)
- ✅ Search engine with fallback mechanisms
- ✅ CloudFlare email decoding
- ✅ Manga page caching for performance
- ✅ Self-search fallback
- ✅ Adult content support

## Build

```bash
cd src/rust/mmrcms
./build.sh mangascan
```

## Configuration

Uses MMRCMS macro-based configuration:

```rust
mmrcms! {
    MMRCMSSource {
        base_url: "http://mangascan-fr.com",
        lang: "fr",
        category_mapper: |idx| {
            String::from(match idx {
                1 => "1",   // Action
                2 => "2",   // Adventure  
                3 => "3",   // Comédie
                // ... 32 total categories
                _ => "",
            })
        },
        ..Default::default()
    }
}
```

## Category Support

Comprehensive genre filtering with 32 categories:
- Action, Adventure, Comédie, Drame
- Ecchi, Erotique, Fantasy, Harem
- Historique, Horreur, Josei, Lemon
- Lime, Mature, Mecha, Mystère
- Psychologique, Reverse Harem, Romance
- School Life, Science Fiction, Seinen
- Shojo, Shoujo Ai, Shounen, Shounen Ai
- Slice of Life, Sports, Supernatural
- Tragedy, Yaoi, Yuri

## Current Status

**⚠️ Website Offline**: MangaScan is currently marked as temporarily offline. This source is maintained for when the site returns online.

### NSFW Notice
This source contains adult content and is properly flagged with `"nsfw": 1` in the configuration.

### When Site Returns
- All MMRCMS template features should function normally
- Extensive category filtering will be available
- Adult content properly handled

This source demonstrates MMRCMS template capabilities with comprehensive genre support and adult content handling, ready for use when the website comes back online.