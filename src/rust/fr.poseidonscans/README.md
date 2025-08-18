# PoseidonScans Source

Custom Aidoku source for the French manga site [poseidonscans.com](https://poseidonscans.com).

## Overview

- **Site**: poseidonscans.com
- **Type**: Custom (Next.js Hybrid)
- **Status**: ✅ Active (Phase 1 Complete)
- **Language**: French
- **NSFW**: Safe content only
- **Package Size**: 105KB

## Features

### Listings
- **Dernières Sorties**: Latest manga with API-based pagination
- **Populaire**: Popular manga from HTML series page

### Hybrid Architecture
- ✅ JSON API for latest manga browsing
- ✅ HTML scraping for popular series
- ✅ Next.js data extraction for manga details
- ✅ Status filtering system (En cours/Terminé/En pause)
- ✅ Client-side search functionality

### Core Functionality
- ✅ Latest manga API with pagination
- ✅ Popular manga HTML parsing
- ✅ Advanced status filtering
- ✅ Search with client-side filtering
- ✅ Manga details with Next.js parsing
- ✅ Chapter lists with date extraction
- ✅ Chapter reading support

## Technical Architecture

### API Endpoints
```
/api/manga/lastchapters    # Latest manga (JSON API)
/series                    # Popular manga (HTML)
/serie/{slug}              # Manga details (Next.js)
/serie/{slug}/chapter/{id} # Chapter pages
```

### Next.js Data Extraction
Supports both Next.js data patterns:
- `__NEXT_DATA__` script extraction
- `self.__next_f.push()` stream parsing

### Status Filtering
Comprehensive status filter implementation:
```json
{
  "type": "select",
  "name": "Status",
  "options": ["Tous les statuts", "En cours", "Terminé", "En pause"]
}
```

Maps to proper `MangaStatus` enum values with client-side filtering.

## Build

```bash
cd src/rust/fr.poseidonscans
./build.sh
```

## Architecture

```
fr.poseidonscans/
├── src/
│   ├── lib.rs           # Main implementation
│   ├── parser.rs        # JSON/HTML/Next.js parsing
│   └── helper.rs        # Utilities
├── res/
│   ├── source.json      # Source metadata
│   ├── filters.json     # Status filter configuration
│   └── Icon.png         # Source icon
├── generate_filters_file.js # Filter extraction tool
└── CLAUDE.md           # Detailed technical documentation
```

## Implementation Highlights

### Phase 1 Achievements (2025-08-15)
- ✅ **Build System**: Package compiles successfully (105KB)
- ✅ **API Integration**: Latest chapters endpoint with pagination
- ✅ **Template Structure**: All required files and functions
- ✅ **Popular Manga**: HTML parsing from `/series` page
- ✅ **Search Functionality**: Client-side filtering
- ✅ **Status Filters**: Complete implementation as requested

### Technical Solutions
- **API Compatibility**: Built with aidoku 0.2.0 for stability
- **Error Handling**: Graceful fallbacks for network failures
- **Anti-Bot Measures**: Modern browser headers and rate limiting

## Special Features

### Date Extraction Strategy
Triple-approach for chapter date parsing:
1. **Relative text**: "22 jours", "1 mois", "3 semaines"
2. **Link association**: Chapter link date attributes
3. **JSON-LD fallback**: Structured data extraction

### Status Filtering Logic
```rust
// Filter mapping
match selected_index {
    1 => filter_by_status(mangas, MangaStatus::Ongoing),
    2 => filter_by_status(mangas, MangaStatus::Completed),
    3 => filter_by_status(mangas, MangaStatus::Hiatus),
    _ => mangas // Show all
}
```

### Cover Image Pattern
```
/api/covers/{slug}.webp
```
Direct API access for optimized cover images.

## Next.js Parsing Examples

### __NEXT_DATA__ Extraction
```javascript
window.__NEXT_DATA__ = {
  "props": {
    "pageProps": {
      "manga": { ... }
    }
  }
}
```

### Stream Parsing
```javascript
self.__next_f.push([1, "data"])
```

## Implementation Notes

### Built from Scratch
Custom implementation required due to PoseidonScans' unique Next.js architecture. Template-based approaches were insufficient for the site's complexity.

### Status Enhancement
Status filtering was specifically implemented as a requested enhancement, providing complete manga status management.

### Future Roadmap
- **Priority 1**: Enhanced Next.js data extraction
- **Priority 2**: Improved manga details metadata
- **Priority 3**: Advanced chapter list parsing
- **Priority 4**: Complex image URL handling

## Maintenance

### Common Issues
- **Empty status results**: Check JSON `status` field mapping
- **Missing covers**: Verify `/api/covers/{slug}.webp` pattern
- **Date parsing failures**: Update French relative date expressions

### Update Points
- **Base URLs**: `lib.rs:15-16` for site changes
- **API endpoints**: Monitor for Next.js route updates
- **Selectors**: HTML structure changes on series page

This source demonstrates successful Next.js site integration with modern API patterns and comprehensive status filtering as specifically requested.