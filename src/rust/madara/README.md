# Madara Template Sources

Template-based Aidoku sources for WordPress Madara CMS manga sites.

## Overview

- **Template Type**: Madara CMS (WordPress)
- **Language**: French
- **Architecture**: Shared template with site-specific configurations
- **Sources**: 4 total (3 active, 1 offline)

## Active Sources

### AstralManga (fr.astralmanga)
- **Site**: astral-manga.fr
- **Status**: ✅ Active
- **Version**: 5
- **Listings**: Tendance, Populaire

### MangaScantrad (fr.mangascantrad) 
- **Site**: manga-scantrad.io
- **Status**: ✅ Active
- **Listings**: Latest, Popular

### MangasOrigines (fr.mangasorigines)
- **Site**: mangas-origines.fr  
- **Status**: ✅ Active
- **Listings**: Latest, Popular

## Offline Sources

### ReaperScansFR (fr.reaperscansfr)
- **Site**: reaper-scans.fr
- **Status**: ❌ Offline
- **Note**: Website temporarily down

## Template Features

### Shared Capabilities
- ✅ WordPress Madara CMS support
- ✅ Advanced search with filters
- ✅ Genre and status filtering
- ✅ Multiple listing types
- ✅ AJAX-based pagination
- ✅ Alternative AJAX endpoints
- ✅ French localization

### Configuration System
Each source uses `MadaraSiteData` configuration:

```rust
template::MadaraSiteData {
    base_url: String::from("https://site-url.com"),
    lang: String::from("fr"),
    description_selector: String::from("div.manga-excerpt p"),
    date_format: String::from("dd/MM/yyyy"),
    status_filter_ongoing: String::from("En cours"),
    status_filter_completed: String::from("Terminé"),
    // Site-specific customizations...
}
```

## Build Commands

```bash
# Build specific source
cd src/rust/madara
./build.sh astralmanga

# Build all Madara sources
./build.sh -a
```

## Architecture

```
madara/
├── template/
│   ├── src/
│   │   ├── lib.rs           # Core template logic
│   │   ├── template.rs      # MadaraSiteData definition
│   │   └── helper.rs        # Utility functions
│   └── Cargo.toml          # Template dependencies
├── sources/
│   ├── astralmanga/        # Site-specific configuration
│   │   ├── src/lib.rs      # get_data() implementation
│   │   ├── res/            # Metadata and filters
│   │   └── Cargo.toml      # Source dependencies
│   ├── mangascantrad/      # Other sources...
│   └── ...
├── res/filters.json        # Shared filter definitions
├── build.sh               # Build script
└── CLAUDE.md              # Detailed documentation
```

## Configuration Fields

### Core Settings
- `base_url`: Site base URL
- `lang`: Language code ("fr")
- `description_selector`: CSS selector for manga descriptions
- `date_format`: Chapter date parsing format

### Localization
- `status_filter_ongoing`: "En cours"
- `status_filter_completed`: "Terminé"
- `popular`: "Populaire"
- `trending`: "Tendance"

### Advanced Options
- `alt_ajax`: Use alternative AJAX endpoint
- Custom status/nsfw parsing closures
- Site-specific CSS selectors

## Filter System

### Shared Filters
- **Genres**: Action, Adventure, Comedy, Drama, Fantasy, Romance, etc.
- **Status**: En cours (Ongoing), Terminé (Completed)
- **Type**: Manga, Manhwa, Manhua
- **Order**: Latest, Popular, Rating, Title

### Site-Specific Filters
Each source can override filter options in their `res/filters.json`.

## Adding New Source

1. **Create source directory**:
   ```bash
   mkdir sources/newsitename
   cp -r template/* sources/newsitename/
   ```

2. **Configure `src/lib.rs`**:
   ```rust
   fn get_data() -> template::MadaraSiteData {
       template::MadaraSiteData {
           base_url: String::from("https://new-site.com"),
           // Site-specific settings...
           ..Default::default()
       }
   }
   ```

3. **Update metadata**: Edit `res/source.json` with site info
4. **Configure filters**: Customize `res/filters.json` if needed
5. **Add icon**: Place site icon in `res/Icon.png`
6. **Test build**: `./build.sh newsitename`

## Maintenance

### Common Updates
- **Domain changes**: Update `base_url` in site configuration
- **Selector updates**: Modify CSS selectors for layout changes
- **Date formats**: Adjust `date_format` for locale changes
- **Status strings**: Update French status text if sites change

### Troubleshooting
- **AJAX issues**: Toggle `alt_ajax` flag for search problems
- **Empty results**: Check CSS selectors in browser dev tools
- **Date parsing**: Verify date format matches site output
- **Status filtering**: Ensure French status strings match site

This template provides a robust foundation for WordPress Madara-based French manga sites with extensive customization options and proven reliability across multiple active sources.