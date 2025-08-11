# CLAUDE.md - MangaStream Template

This file provides guidance for working with MangaStream template sources in this repository.

## MangaStream Template Architecture

The MangaStream template supports manga streaming framework sites with a shared codebase and site-specific configurations.

### Current Sources
- **sushiscan**: SushiScan v6 (sushiscan.net) - New domain, `alt_pages: true`
- **sushiscans**: Sushi Scans (sushiscan.fr) - Legacy domain

## Build Commands

```bash
# Build specific source
./build.sh sushiscan

# Build all MangaStream sources
./build.sh -a
```

## Source Configuration

Each source is configured through `get_instance()` function returning `MangaStreamSource`:

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
        alt_pages: true,
        ..Default::default()
    }
}
```

### Key Configuration Fields
- `base_url`: Site base URL
- `listing`: Available manga listings in French
- `status_options`: Status filter options
- `status_options_search`: Search URL parameters for status
- `type_options_search`: Search URL parameters for manga types
- `traverse_pathname`: Path for manga catalog/browse
- `last_page_text`: Text indicating pagination end
- `manga_details_*`: CSS selectors for manga details parsing
- `chapter_date_format`: Date format for chapter dates
- `alt_pages`: Enable alternative page parsing (set to `true` for SushiScan v6)

## Adding New MangaStream Source

1. **Create source directory**: `sources/newsitename/`
2. **Copy template structure**:
   ```bash
   cp -r template/ sources/newsitename/
   ```
3. **Configure `src/lib.rs`**: Update `get_instance()` with site settings
4. **Update `res/source.json`**: Set source metadata
5. **Configure `res/filters.json`**: Define search filters
6. **Add icon**: Place `Icon.png` in `res/`
7. **Test build**: `./build.sh newsitename`

## Common Issues

- **Domain changes**: Update `base_url` when sites change domains
- **Alt pages**: Toggle `alt_pages` if chapter pages fail to load (SushiScan v6 requires `alt_pages: true`)
- **Pagination**: Adjust `last_page_text` for different pagination systems
- **Selectors**: Modify CSS selectors when site structure changes
- **Date formats**: Update `chapter_date_format` for locale-specific dates

## Recent Changes

- **2025-08-10**: MAJOR DISCOVERY: SushiScan uses completely different HTML structure
  - **Browser inspection revealed**: SushiScan does NOT use MangaStream template structure at all
  - **Wrong selectors identified**: `.listupd .bsx` does not exist on SushiScan - was the root cause!
  - **Real SushiScan structure**: `<a href="/catalogue/"><div class="limit"><img src="..."></div></a>`
  - **Corrected selectors**: `manga_selector: "a[href*='/catalogue/']"` and `manga_title: ".tt"`
  - **Perfect image URLs**: Direct URLs like `https://sushiscan.net/wp-content/uploads/MangaCover.jpg`
  - **Result**: Should finally display manga covers correctly with proper selectors

- **2025-08-10**: CRITICAL FIX: Resolved chapter page loading issues (black screen with reload)
  - **Simplified image headers**: Removed aggressive headers that blocked chapter page images
  - **Essential headers only**: Keep Accept, Accept-Language, Referer, and User-Agent for compatibility
  - **Result**: Chapter pages should now load properly instead of showing black screen with "Recharger"

- **2025-08-10**: CRITICAL FIX: Enhanced image cover loading for manga listings
  - **Improved image selectors**: Added support for `.bs img` and `a img` selectors for better coverage
  - **Enhanced lazy loading**: Added support for `data-wpfc-original-src` and `data-original` attributes
  - **Better URL validation**: Added checks to ensure valid image URLs with file extensions
  - **Result**: Manga covers should now display properly instead of appearing as black rectangles

- **2025-08-10**: CRITICAL FIX: Balanced anti-bot detection with functionality
  - **Headers simplified**: Removed aggressive headers that blocked manga list loading
  - **User-Agent reverted**: Back to Safari to avoid Chrome-specific detection
  - **Essential headers kept**: Only Accept and Accept-Language for functionality
  - **JSON parsing preserved**: Kept robust ts_reader.run() extraction for chapters

- **2025-08-10**: CRITICAL FIX: Robust JSON parsing for ts_reader.run() 
  - **Root cause identified**: Complex JSON parsing was failing silently, causing empty page lists
  - **MCP browser testing**: Confirmed 17 images exist in ts_reader.run() but only 1 in DOM
  - **Simple regex approach**: Direct extraction of images array instead of full JSON parsing
  - **Robust URL cleaning**: Better handling of quotes, backslashes, and escaping
  - **.webp validation**: Ensures only valid image URLs are processed
  - **Result**: SushiScan chapters should now load all 17 pages instead of failing

- **2025-08-10**: Critical HTTP header improvements to bypass anti-bot protections
  - **Modern User-Agent**: Updated to Chrome 131.0.0.0 from Safari 18.3 for better compatibility
  - **Complete browser headers**: Added Accept, Accept-Language, Accept-Encoding, DNT, Connection, Upgrade-Insecure-Requests
  - **Anti-bot bypass**: Full browser emulation to counter SushiScan's detection systems
  - **All requests covered**: Updated manga listing, details, chapter list, and page list requests
  - **Root cause identified**: Browser testing showed CSS selectors work perfectly - issue was HTTP-level blocking

- **2025-08-09**: Comprehensive fix for SushiScan chapter loading issues
  - **Reordered parsing logic**: SushiScan `ts_reader.run()` format attempted first
  - **Robust error handling**: Replaced panic-prone `?` operators with `if let Ok` checks
  - **Smart fallback**: Only attempts original `:[{"s` pattern if no pages found  
  - **URL correction**: Added automatic trailing slash for proper chapter URLs
  - **JSON parsing**: Full support for `ts_reader.run({"sources":[{"source":"Server 1","images":[...]}]})` structure
  - **Backward compatibility**: Maintains support for existing MangaStream sites

## Version Management

- Use versioned source IDs (e.g., `fr.sushiscan.v6`) when major changes occur
- Update version number in `source.json` for incremental updates