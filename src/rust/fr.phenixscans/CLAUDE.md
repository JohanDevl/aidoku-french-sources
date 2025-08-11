# CLAUDE.md - PhenixScans Custom Source

This file provides guidance for working with the PhenixScans custom source implementation.

## Source Overview

**PhenixScans** is a custom implementation for the French manga site `phenix-scans.com`. This source is unique as it uses **JSON API endpoints** rather than HTML parsing, making it more reliable and faster than traditional scraping approaches.

### Site Information
- **URL**: https://phenix-scans.com
- **API URL**: https://phenix-scans.com/api
- **Language**: French (fr)
- **NSFW**: Safe content only
- **Status**: ✅ Active

## Build Commands

```bash
# Build the source
./build.sh

# Or using cargo directly
cargo +nightly build --release
```

## Architecture

The source uses a **JSON API-based architecture** with clean separation of concerns:

### Core Files
- `src/lib.rs`: Main source implementation with API calls
- `src/parser.rs`: JSON response parsing and data transformation
- `src/helper.rs`: Utility functions for URL encoding and string manipulation
- `res/source.json`: Source metadata and configuration
- `res/filters.json`: Search and filter definitions

### API Endpoints Used

```rust
// Base URLs
pub static BASE_URL: &str = "https://phenix-scans.com";
pub static API_URL: &str = "https://phenix-scans.com/api";

// Main endpoints
/front/manga              // Browse/search with filters
/front/manga/search       // Text search
/front/homepage           // Listings (latest/popular)  
/front/manga/{id}         // Manga details & chapters
/front/manga/{id}/chapter/{chapter_id} // Chapter pages
```

## Key Features

### Dual Search System
- **Filter-based browse**: `/front/manga` with advanced filtering
- **Text search**: `/front/manga/search` for title queries

### Advanced Filtering
- **Status**: Ongoing, Completed, Hiatus
- **Type**: Dynamic type selection from API
- **Sorting**: By rating, update date, title, chapter count
- **Genres**: Multiple genre selection support

### Listing Support
- **Dernières Sorties**: Latest releases with pagination
- **Populaire**: Top-rated manga (no pagination)

## JSON Response Structures

### Manga List Response
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

### Listings Response
```json
{
  "latest": [...],  // Latest releases
  "top": [...]      // Popular manga
}
```

### Manga Details Response
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

## Common Maintenance Tasks

### API Endpoint Changes
If PhenixScans updates their API:
1. **Update base URLs** in `lib.rs:15-16`
2. **Check endpoint paths** in each function
3. **Verify JSON response structure** in `parser.rs`

### New Filter Options
When new filters become available:
1. **Update `res/filters.json`** with new options
2. **Add parsing logic** in `get_manga_list()` 
3. **Test filter combinations**

### Response Format Changes
If JSON structure changes:
1. **Update parser functions** in `parser.rs`
2. **Check field mappings** (slug, title, coverImage, etc.)
3. **Test all data extraction paths**

## Advantages of API-Based Approach

- **Reliability**: Less prone to breaking than HTML parsing
- **Performance**: Faster than scraping with CSS selectors
- **Maintainability**: Cleaner code with JSON parsing
- **Features**: Rich filtering and search capabilities

## Debugging Tips

- **API errors**: Check response status and JSON validity
- **Empty results**: Verify API endpoints and parameters
- **Missing images**: Check `coverImage` path construction
- **Search issues**: Test both search methods (filter vs text)
- **Date parsing**: Ensure ISO date format handling in `parse_date_string()`

## Error Handling

The source includes robust error handling for:
- Network failures
- Invalid JSON responses  
- Missing required fields
- API rate limiting (if implemented)

## Special Features

- **Skip unknown manga**: Filters out items with `slug: "unknown"`
- **Dynamic type options**: Reads available types from filter configuration
- **Image URL construction**: Builds full URLs for cover images
- **Date normalization**: Converts API dates to Aidoku format