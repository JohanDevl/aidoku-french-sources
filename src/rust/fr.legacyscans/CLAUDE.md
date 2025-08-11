# CLAUDE.md - LegacyScans Custom Source

This file provides guidance for working with the LegacyScans custom source implementation.

## Source Overview

**LegacyScans** is a custom implementation for the French manga site `legacy-scans.com`. This source uses a **hybrid architecture** combining JSON API endpoints for browsing/search with traditional HTML parsing for manga details and chapters.

### Site Information
- **URL**: https://legacy-scans.com
- **API URL**: https://api.legacy-scans.com
- **Language**: French (fr)
- **NSFW**: Safe content only
- **Status**: ❌ Offline (Website temporarily down)

## Build Commands

```bash
# Build the source
./build.sh

# Or using cargo directly
cargo +nightly build --release
```

## Hybrid Architecture

The source combines two approaches for optimal performance and reliability:

### JSON API Endpoints (Browse & Search)
```rust
pub static API_URL: &str = "https://api.legacy-scans.com";

// Search endpoints
/misc/home/search              // Title search
/misc/comic/search/query       // Advanced filtering
/misc/comic/home/updates       // Latest releases
/misc/views/daily              // Daily popular
/misc/views/monthly            // Monthly popular  
/misc/views/all                // All-time popular
```

### HTML Parsing (Details & Chapters)
```rust
pub static BASE_URL: &str = "https://legacy-scans.com";

// HTML endpoints
/comics/{id}                   // Manga details & chapters
/comics/{id}/chapter/{chap_id} // Chapter pages
```

## Core Files

- `src/lib.rs`: Main implementation with dual API/HTML approach
- `src/parser.rs`: JSON + HTML parsing logic
- `src/helper.rs`: URL encoding utilities
- `res/source.json`: Source metadata with multiple listings
- `res/filters.json`: Advanced filtering options

## Key Features

### Pagination System
LegacyScans uses **range-based pagination** instead of simple page numbers:

```rust
let manga_per_page = 20;
let end = manga_per_page * page;
let start = (end - manga_per_page) + 1;
// Request: ?start=1&end=20, then ?start=21&end=40
```

### Dual Search Methods
- **Title search**: `/misc/home/search?title=query`  
- **Advanced search**: `/misc/comic/search/query` with filters

### Rich Listings
- **Dernières Sorties**: Latest releases (paginated)
- **Populaire (Jour)**: Daily popular
- **Populaire (Mois)**: Monthly popular  
- **Populaire (Tous)**: All-time popular

### Advanced Filtering
- **Status**: Multiple status options from filter config
- **Type**: Multiple type options from filter config
- **Order**: Sorting options (ascending/descending)
- **Genres**: Multi-select genre filtering

## JSON Response Structures

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

### Comics List
```json
{
  "comics": [
    {
      "slug": "manga-id",
      "title": "Manga Title", 
      "cover": "/path/to/cover.jpg"
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

## Common Maintenance Tasks

### API Endpoint Updates
If API structure changes:
1. **Check endpoint URLs** in `lib.rs:73,77,90-96`
2. **Verify JSON response format** in `parser.rs`
3. **Test pagination parameters** (start/end ranges)

### HTML Structure Changes  
If site redesign affects manga pages:
1. **Update CSS selectors** in HTML parsing functions
2. **Check manga details extraction** 
3. **Verify chapter list parsing**
4. **Test page URL extraction**

### Filter Configuration
When site adds new filter options:
1. **Update `res/filters.json`** with new options
2. **Add parsing logic** in `get_manga_list()` filters loop
3. **Test filter parameter encoding**

## Debugging Tips

### API Issues
- **Check API availability**: `curl https://api.legacy-scans.com/misc/views/all`
- **Verify JSON structure**: Use browser dev tools or Postman
- **Range pagination**: Ensure start/end calculations are correct

### HTML Parsing Issues  
- **Site downtime**: LegacyScans may be temporarily offline
- **Selector updates**: Check if HTML structure changed
- **Encoding issues**: Verify URL encoding for search terms

### Performance Tips
- **API preference**: Use API endpoints when possible (faster than HTML)
- **Range requests**: Optimize pagination for better loading
- **Caching**: Consider caching popular lists locally

## Special Considerations

### Site Status
LegacyScans is currently marked as **offline** in the README. When working on this source:
- **Check site availability** before making changes
- **Test against live site** if it comes back online
- **Consider archival** if permanently down

### Hybrid Benefits
- **Speed**: API calls are faster than HTML parsing
- **Reliability**: HTML fallback when API fails
- **Features**: Rich filtering from API + detailed parsing from HTML

### Headers Required
Some endpoints require the `Referer` header:
```rust
.header("Referer", String::from(BASE_URL).as_str())
```

## Error Handling

The source handles:
- API endpoint failures
- HTML parsing errors
- Network timeouts
- Invalid JSON responses
- Missing manga data gracefully