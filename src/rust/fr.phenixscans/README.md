# PhenixScans Source

Custom Aidoku source for the French manga site [phenix-scans.com](https://phenix-scans.com).

## Overview

- **Site**: phenix-scans.com
- **API**: phenix-scans.com/api
- **Type**: Custom (Pure JSON API)
- **Status**: ✅ Active
- **Language**: French
- **NSFW**: Safe content only

## Features

### Listings
- **Dernières Sorties**: Latest releases with pagination
- **Populaire**: Top-rated manga (no pagination)

### Modern API Architecture
- ✅ Pure JSON API implementation (no HTML scraping)
- ✅ Advanced filtering system with multiple criteria
- ✅ Dual search methods (filter-based + text search)
- ✅ Rich manga metadata extraction
- ✅ Reliable pagination system

### Search & Browse
- ✅ Text search via dedicated endpoint
- ✅ Filter-based browsing with advanced options
- ✅ Status filtering (Ongoing, Completed, Hiatus)
- ✅ Type and genre filtering
- ✅ Multiple sorting options

## Technical Architecture

### API Endpoints
```
/front/manga              # Browse with filters
/front/manga/search       # Text search
/front/homepage           # Listings (latest/popular)
/front/manga/{id}         # Manga details & chapters
/front/manga/{id}/chapter/{chapter_id} # Chapter pages
```

### JSON Response Handling
All data comes from clean JSON APIs with structured responses:

```json
{
  "pagination": { "current": 1, "total": 10 },
  "mangas": [
    {
      "slug": "manga-id",
      "title": "Manga Title",
      "coverImage": "/path/to/cover.jpg"
    }
  ]
}
```

## Build

```bash
cd src/rust/fr.phenixscans
./build.sh
```

## Architecture

```
fr.phenixscans/
├── src/
│   ├── lib.rs           # Main API implementation
│   ├── parser.rs        # JSON response parsing
│   └── helper.rs        # URL utilities
├── res/
│   ├── source.json      # Source metadata
│   ├── filters.json     # Advanced filter definitions
│   └── Icon.png         # Source icon
├── generate_filters_file.js # Dynamic filter generation
└── CLAUDE.md           # Detailed technical documentation
```

## Advanced Features

### Dual Search System
- **Filter Browse**: `/front/manga` with comprehensive filtering
- **Text Search**: `/front/manga/search` for direct title queries

### Rich Filtering
- **Status**: Ongoing, Completed, Hiatus
- **Type**: Dynamic type selection from API
- **Genres**: Multiple genre selection
- **Sorting**: Rating, update date, title, chapter count

### Dynamic Filters
Includes JavaScript tool for extracting filter options directly from the live site:
```bash
node generate_filters_file.js
```

## API Benefits

### Reliability
- **No CSS selectors**: Immune to visual redesigns
- **Structured data**: Consistent JSON format
- **Error handling**: Clear API error responses

### Performance
- **Faster loading**: JSON is lighter than HTML
- **Efficient parsing**: Direct data access
- **Reduced requests**: Single API call per operation

### Maintainability
- **Clean code**: Simple JSON parsing
- **Future-proof**: API less likely to break
- **Rich features**: Native filtering and search

## Response Examples

### Manga Details
```json
{
  "slug": "manga-id",
  "title": "Manga Title",
  "description": "Description...",
  "genres": ["Action", "Adventure"],
  "chapters": [
    {
      "slug": "chapter-id",
      "title": "Chapter 1",
      "createdAt": "2023-01-01T00:00:00.000Z"
    }
  ]
}
```

### Listings
```json
{
  "latest": [...],  // Latest releases
  "top": [...]      // Popular manga
}
```

## Special Features

- **Unknown manga filtering**: Automatically skips items with `slug: "unknown"`
- **Dynamic type options**: Reads available types from configuration
- **Image URL construction**: Builds complete URLs for covers
- **ISO date handling**: Proper date parsing from API

This source represents the ideal modern approach to Aidoku source development, leveraging clean JSON APIs for maximum reliability and feature richness.