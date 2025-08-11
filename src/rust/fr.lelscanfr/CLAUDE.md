# CLAUDE.md - LelscanFR Custom Source

This file provides guidance for working with the LelscanFR custom source implementation.

## Source Overview

**LelscanFR** is a custom implementation for the French manga site `lelscanfr.com`. This source uses custom parsing logic tailored specifically to their unique website structure.

### Site Information
- **URL**: https://lelscanfr.com
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

The source follows a custom implementation pattern with three main modules:

### Core Files
- `src/lib.rs`: Main source implementation with Aidoku entry points
- `src/parser.rs`: HTML parsing logic for manga data extraction  
- `src/helper.rs`: Utility functions for URL encoding and string manipulation
- `res/source.json`: Source metadata and configuration
- `res/filters.json`: Search and filter definitions

### Main Functions
- `get_manga_list()`: Search and filter manga listings
- `get_manga_details()`: Extract detailed manga information
- `get_chapter_list()`: Parse available chapters (with pagination support)
- `get_page_list()`: Extract chapter page URLs

## Key Features

### Search & Filtering
- **Title search**: URL-encoded title query parameter
- **Genre filtering**: Multiple genre selection support
- **Type filtering**: Manga, Manhua, Manhwa, Bande-dessinée
- **Status filtering**: En cours (Ongoing), Terminé (Completed)

### Pagination Handling
The source handles paginated chapter lists automatically:
```rust
if !html.select(".pagination").text().read().is_empty() {
    let nb_pages = html.select(".pagination-link").last()
        .previous().expect("last page")
        .attr("onclick").read()
        .chars().rev().nth(1).unwrap()
        .to_digit(10).unwrap();
    // Fetch all pages...
}
```

### CSS Selectors Used
- Manga cards: `div[id=card-real]`
- Pagination: `.pagination-disabled[aria-label~=Next]`
- Manga details: `main img`, `span:contains(Auteur)+span`
- Chapter lists: Custom parsing with pagination detection
- Images: `data-src` attribute for lazy-loaded images

## Common Maintenance Tasks

### Updating Selectors
When LelscanFR changes their site structure:

1. **Manga list parsing** (`parser.rs:14`):
   - Update `div[id=card-real]` if card structure changes
   - Check image selector `img[data-src]` vs `img[src]`

2. **Pagination detection** (`parser.rs:39`):
   - Verify `.pagination-disabled[aria-label~=Next]` selector
   - Update pagination logic if needed

3. **Manga details** (`parser.rs:47+`):
   - Update author selector `span:contains(Auteur)+span`
   - Check cover image selector `main img`

### URL Structure Changes
If LelscanFR changes their URL patterns:
- Update `BASE_URL` constant in `lib.rs:16`
- Modify ID extraction logic in `parser.rs:21`
- Update URL construction in `get_chapter_list()`

### Filter Configuration
Update `res/filters.json` if site adds new:
- Genre categories
- Status options  
- Type classifications

## Debugging Tips

- **Network issues**: Check if `BASE_URL` is current
- **Empty results**: Verify CSS selectors in browser dev tools
- **Pagination errors**: Test with manga having many chapters
- **Encoding issues**: Use `helper::urlencode()` for query parameters