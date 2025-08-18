# SushiScan v6 Source

MangaStream template-based source for [sushiscan.net](https://sushiscan.net) (New domain).

## Overview

- **Site**: sushiscan.net (New domain v6)
- **Template**: MangaStream (Modified)
- **Status**: ✅ Active
- **Language**: French
- **Version**: 4

## Features

### Listings
- **Populaire**: Popular manga
- **Dernières**: Latest releases
- **Nouveau**: New manga

### Advanced Parsing
- ✅ Alternative page parsing (`alt_pages: true`)
- ✅ JavaScript `ts_reader.run()` extraction
- ✅ Anti-bot header simulation
- ✅ Robust JSON parsing with fallbacks
- ✅ Complete browser emulation

## Build

```bash
cd src/rust/mangastream
./build.sh sushiscan
```

## Technical Configuration

### Key Settings
```rust
MangaStreamSource {
    base_url: String::from("https://sushiscan.net"),
    listing: ["Dernières", "Populaire", "Nouveau"],
    status_options: ["En Cours", "Terminé", "Abandonné", "En Pause", ""],
    traverse_pathname: "catalogue",
    last_page_text: "Suivant",
    language: "fr",
    alt_pages: true,  // Critical for v6 structure
    ..Default::default()
}
```

## Recent Critical Fixes (2025-08-10)

### Major Discovery
- **Wrong Template**: SushiScan does NOT use standard MangaStream structure
- **Corrected Selectors**: Fixed `manga_selector` to `a[href*='/catalogue/']`
- **Real Structure**: `<div class="limit"><img></div>` inside catalog links

### JSON Parsing Enhancement
- **Robust Extraction**: Simplified `ts_reader.run()` parsing
- **Regex Approach**: Direct image array extraction
- **Fallback Support**: Multiple parsing strategies

### Anti-Bot Improvements
- **Modern Headers**: Chrome 131.0.0.0 User-Agent
- **Complete Simulation**: Full browser header set
- **Request Coverage**: All endpoints properly simulated

## Special Features

### Alternative Pages (`alt_pages: true`)
SushiScan v6 requires special page parsing due to unique HTML structure.

### JavaScript Extraction
Advanced parsing of `ts_reader.run()` JSON format:
```javascript
ts_reader.run({"sources":[{"source":"Server 1","images":[...]}]})
```

### Image URL Handling
Supports multiple lazy-loading patterns:
- `data-wpfc-original-src`
- `data-original`
- `data-src`
- Standard `src`

This source represents the cutting-edge of MangaStream template evolution, specifically adapted for SushiScan v6's unique architecture with robust anti-bot measures and advanced JSON extraction.