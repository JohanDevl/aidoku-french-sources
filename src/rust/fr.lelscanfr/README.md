# LelscanFR Source

Custom Aidoku source for the French manga site [lelscanfr.com](https://lelscanfr.com).

## Overview

- **Site**: lelscanfr.com
- **Type**: Custom (HTML Scraping)
- **Status**: ✅ Active
- **Language**: French
- **NSFW**: Safe content only
- **Version**: 3

## Features

### Search & Browse
- ✅ Title search with URL encoding
- ✅ Genre filtering with multi-select support
- ✅ Type filtering (Manga, Manhua, Manhwa, Bande-dessinée)
- ✅ Status filtering (En cours, Terminé)
- ✅ Advanced pagination handling

### Core Functionality
- ✅ Manga listings with card-based layout
- ✅ Detailed manga information extraction
- ✅ Chapter lists with automatic pagination
- ✅ Chapter reading with lazy-loaded images
- ✅ Robust error handling

## Technical Implementation

### HTML Parsing Strategy
Custom CSS selectors tailored to LelscanFR's unique structure:

```css
div[id=card-real]                    /* Manga cards */
.pagination-disabled[aria-label~=Next] /* Pagination detection */
img[data-src]                        /* Lazy-loaded images */
span:contains(Auteur)+span          /* Author extraction */
```

### Smart Pagination
Automatically detects and handles paginated chapter lists:
```rust
let nb_pages = html.select(".pagination-link").last()
    .previous().expect("last page")
    .attr("onclick").read()
    .chars().rev().nth(1).unwrap()
    .to_digit(10).unwrap();
```

### Image Handling
Supports both standard and lazy-loaded images:
- Primary: `data-src` attribute (lazy loading)
- Fallback: `src` attribute (direct loading)

## Build

```bash
cd src/rust/fr.lelscanfr
./build.sh
```

## Architecture

```
fr.lelscanfr/
├── src/
│   ├── lib.rs           # Main implementation
│   ├── parser.rs        # HTML parsing logic
│   └── helper.rs        # URL encoding utilities
├── res/
│   ├── source.json      # Source metadata (v3)
│   ├── filters.json     # Search and filter definitions
│   └── Icon.png         # Source icon
└── CLAUDE.md           # Detailed technical documentation
```

## Key Selectors

### Manga Discovery
- **Cards**: `div[id=card-real]` - Main manga card container
- **Images**: `img[data-src]` or `img[src]` - Cover images
- **Links**: Extracted from card structure

### Pagination
- **Detection**: `.pagination-disabled[aria-label~=Next]`
- **Navigation**: `.pagination-link` elements with onclick handlers

### Manga Details
- **Cover**: `main img` - Primary cover image
- **Author**: `span:contains(Auteur)+span` - Author name extraction
- **Description**: Custom parsing from page structure

## Filter Configuration

### Available Filters
- **Genres**: Multiple selection from comprehensive list
- **Status**: En cours (Ongoing), Terminé (Completed)
- **Type**: Manga, Manhua, Manhwa, Bande-dessinée

### URL Parameter Mapping
All filters are properly URL-encoded and mapped to LelscanFR's expected parameter format.

## Maintenance Notes

### Selector Updates
When site structure changes, update these key selectors:
1. **Manga cards**: `div[id=card-real]`
2. **Pagination**: `.pagination-disabled[aria-label~=Next]`
3. **Images**: `img[data-src]` vs `img[src]` preference

### URL Changes
Monitor for changes in:
- Base URL structure
- Filter parameter naming
- Pagination URL patterns

This source demonstrates solid HTML scraping techniques with robust pagination handling and comprehensive filter support for LelscanFR's custom site structure.