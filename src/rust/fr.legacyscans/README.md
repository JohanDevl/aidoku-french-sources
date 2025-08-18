# LegacyScans Source

Custom Aidoku source for the French manga site [legacy-scans.com](https://legacy-scans.com).

## Overview

- **Site**: legacy-scans.com
- **API**: api.legacy-scans.com
- **Type**: Custom (Hybrid API + HTML)
- **Status**: ❌ Offline (Website temporarily down)
- **Language**: French
- **NSFW**: Safe content only

## Features

### Listings
- **Dernières Sorties**: Latest releases with pagination
- **Populaire (Jour)**: Daily popular manga
- **Populaire (Mois)**: Monthly popular manga
- **Populaire (Tous)**: All-time popular manga

### Hybrid Architecture
- ✅ JSON API for browsing and search (faster)
- ✅ HTML parsing for manga details and chapters (comprehensive)
- ✅ Advanced filtering with multiple criteria
- ✅ Range-based pagination system

### Search & Browse
- ✅ Title search via API endpoint
- ✅ Advanced filtering (status, type, genres)
- ✅ Multiple sorting options
- ✅ Efficient pagination

## Technical Architecture

### API Endpoints
```
/misc/home/search              # Title search
/misc/comic/search/query       # Advanced filtering
/misc/comic/home/updates       # Latest releases
/misc/views/daily              # Daily popular
/misc/views/monthly            # Monthly popular
/misc/views/all                # All-time popular
```

### HTML Endpoints
```
/comics/{id}                   # Manga details & chapters
/comics/{id}/chapter/{chap_id} # Chapter pages
```

### Unique Pagination
Range-based pagination instead of simple page numbers:
```
?start=1&end=20    # First page (20 items)
?start=21&end=40   # Second page (20 items)
```

## Build

```bash
cd src/rust/fr.legacyscans
./build.sh
```

## Architecture

```
fr.legacyscans/
├── src/
│   ├── lib.rs           # Main implementation with dual approach
│   ├── parser.rs        # JSON + HTML parsing logic
│   └── helper.rs        # URL encoding utilities
├── res/
│   ├── source.json      # Source metadata with 4 listings
│   ├── filters.json     # Advanced filtering options
│   └── Icon.png         # Source icon
└── CLAUDE.md           # Detailed technical documentation
```

## API Response Examples

### Search Results
```json
{
  "results": [
    {
      "slug": "manga-id",
      "title": "Manga Title"
    }
  ]
}
```

### Popular Lists
```json
[
  {
    "slug": "manga-id",
    "title": "Manga Title",
    "cover": "/path/to/cover.jpg"
  }
]
```

## Current Status

**⚠️ Website Offline**: LegacyScans is currently marked as temporarily offline. This source is maintained for when the site returns online.

### When Site Returns
- All API endpoints should function normally
- Hybrid architecture provides maximum compatibility
- No code changes expected due to robust implementation

## Technical Benefits

- **Speed**: API calls are much faster than HTML scraping
- **Reliability**: HTML fallback when API endpoints fail
- **Features**: Rich filtering capabilities from API
- **Maintenance**: Dual approach reduces breakage risk

This source demonstrates optimal hybrid architecture, combining the speed of API access with the robustness of HTML parsing fallbacks.