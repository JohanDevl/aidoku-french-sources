# MangaStream Template Sources

Template-based Aidoku sources for MangaStream framework manga sites.

## Overview

- **Template Type**: MangaStream Framework
- **Language**: French  
- **Architecture**: Shared template with instance configurations
- **Sources**: 2 variants of SushiScan

## Active Sources

### SushiScan v6 (fr.sushiscan)
- **Site**: sushiscan.net (New domain)
- **Status**: ✅ Active 
- **Version**: 4
- **Listings**: Populaire, Dernières, Nouveau
- **Special**: `alt_pages: true` for v6 structure

### SushiScans Legacy (fr.sushiscans)
- **Site**: sushiscan.fr (Legacy domain)
- **Status**: ✅ Active
- **Listings**: Standard MangaStream layout

## Template Features

### Core Capabilities
- ✅ MangaStream framework support
- ✅ Advanced search and filtering
- ✅ Multiple listing types (Popular, Latest, New)
- ✅ Genre and status filtering
- ✅ Alternative page parsing (v6 support)
- ✅ Anti-bot header simulation
- ✅ JavaScript JSON extraction

### Advanced Parsing
- **Chapter Pages**: `ts_reader.run()` JSON extraction
- **Alternative Pages**: Support for different HTML structures  
- **Robust Headers**: Complete browser simulation
- **Image Loading**: Multiple lazy-loading attribute support

## Build Commands

```bash
# Build specific source
cd src/rust/mangastream
./build.sh sushiscan

# Build all MangaStream sources
./build.sh -a
```

## Architecture

```
mangastream/
├── template/
│   ├── src/
│   │   ├── lib.rs           # Core template logic
│   │   ├── template.rs      # MangaStreamSource definition
│   │   └── helper.rs        # Utility functions
│   └── Cargo.toml          # Template dependencies
├── sources/
│   ├── sushiscan/          # v6 configuration
│   │   ├── src/lib.rs      # get_instance() implementation
│   │   ├── res/            # Metadata and filters
│   │   └── Cargo.toml      # Source dependencies
│   ├── sushiscans/         # Legacy configuration
│   └── ...
├── res/filters.json        # Shared filter definitions
├── build.sh               # Build script
└── CLAUDE.md              # Detailed documentation
```

## Configuration System

Each source implements `get_instance()` returning `MangaStreamSource`:

```rust
fn get_instance() -> MangaStreamSource {
    MangaStreamSource {
        base_url: String::from("https://sushiscan.net"),
        listing: ["Dernières", "Populaire", "Nouveau"],
        status_options: ["En Cours", "Terminé", "Abandonné", "En Pause", ""],
        traverse_pathname: "catalogue",
        last_page_text: "Suivant", 
        manga_details_categories: ".seriestugenre a",
        chapter_date_format: "MMMM dd, yyyy",
        language: "fr",
        alt_pages: true,  // Critical for SushiScan v6
        ..Default::default()
    }
}
```

## Key Configuration Fields

### Core Settings
- `base_url`: Site base URL
- `listing`: Available manga listings in French
- `language`: Language code ("fr")
- `traverse_pathname`: Catalog/browse path

### Parsing Configuration
- `manga_details_*`: CSS selectors for details extraction
- `chapter_date_format`: Date parsing format
- `last_page_text`: Pagination end indicator
- `alt_pages`: Alternative page parsing mode

### Filter Options
- `status_options`: Available status filters
- `status_options_search`: URL parameters for status
- `type_options_search`: URL parameters for types

## Recent Critical Fixes

### 2025-08-10: Major Discovery & Fixes

#### Wrong Template Identification
- **Discovery**: SushiScan does NOT use standard MangaStream structure
- **Root Cause**: Incorrect selectors like `.listupd .bsx` don't exist
- **Real Structure**: `<a href="/catalogue/"><div class="limit"><img></div></a>`
- **Fix**: Corrected selectors to `a[href*='/catalogue/']` and `.tt`

#### Chapter Page Loading
- **Issue**: Black screen with "Recharger" button
- **Cause**: Aggressive headers blocking image requests
- **Fix**: Simplified headers to essential only (Accept, Accept-Language, Referer, User-Agent)

#### JSON Parsing Enhancement
- **Issue**: `ts_reader.run()` complex JSON parsing failures
- **Solution**: Simple regex extraction instead of full JSON parsing
- **Result**: Reliable extraction of all chapter images

#### Anti-Bot Improvements  
- **Headers**: Complete browser emulation with Chrome 131.0.0.0
- **Coverage**: All request types (listings, details, chapters, pages)
- **Balance**: Functionality preserved while bypassing detection

## Technical Highlights

### JavaScript Extraction
Advanced `ts_reader.run()` JSON parsing:
```javascript
ts_reader.run({"sources":[{"source":"Server 1","images":[...]}]})
```

### Anti-Bot Measures
Complete browser header simulation:
```rust
.header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
.header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
// ... additional headers
```

### Image URL Handling
Multiple lazy-loading attribute support:
- `data-wpfc-original-src`
- `data-original`
- `data-src`
- Standard `src`

## Adding New Source

1. **Create source directory**:
   ```bash
   mkdir sources/newsitename
   cp -r template/* sources/newsitename/
   ```

2. **Configure `src/lib.rs`**:
   ```rust
   fn get_instance() -> MangaStreamSource {
       MangaStreamSource {
           base_url: String::from("https://new-site.com"),
           // Site-specific settings...
           ..Default::default()
       }
   }
   ```

3. **Test site structure**: Verify if `alt_pages: true` is needed
4. **Update metadata**: Edit `res/source.json`
5. **Configure filters**: Customize `res/filters.json`
6. **Add icon**: Place site icon in `res/Icon.png`
7. **Test build**: `./build.sh newsitename`

## Troubleshooting

### Common Issues
- **Empty manga lists**: Check CSS selectors against actual HTML
- **Chapter loading failures**: Verify JSON extraction patterns
- **Image loading**: Test lazy-loading attribute support
- **Anti-bot blocks**: Update User-Agent and headers

### Debug Steps
1. **Browser inspection**: Verify selectors work in dev tools
2. **Network monitoring**: Check for blocked requests
3. **JavaScript console**: Test `ts_reader.run()` patterns
4. **Alternative parsing**: Toggle `alt_pages` flag

This template has been battle-tested with SushiScan's complex requirements and provides robust support for MangaStream framework sites with advanced anti-bot measures and flexible parsing options.