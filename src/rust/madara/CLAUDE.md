# CLAUDE.md - Madara Template

This file provides guidance for working with Madara template sources in this repository.

## Madara Template Architecture

The Madara template supports WordPress Madara CMS-based manga sites with a shared codebase and site-specific configurations.

### Current Sources
- **astralmanga**: Astral Manga (astral-manga.fr)
- **mangascantrad**: MangaScantrad (manga-scantrad.io)  
- **mangasorigines**: MangasOrigines (mangas-origines.fr)
- **reaperscansfr**: ReaperScansFR (reaper-scans.fr) - Currently offline

## Build Commands

```bash
# Build specific source
./build.sh astralmanga

# Build all Madara sources
./build.sh -a
```

## Source Configuration

Each source is configured through `get_data()` function returning `MadaraSiteData`:

```rust
fn get_data() -> template::MadaraSiteData {
    template::MadaraSiteData {
        base_url: String::from("https://site-url.com"),
        lang: String::from("fr"),
        description_selector: String::from("div.manga-excerpt p"),
        date_format: String::from("dd/MM/yyyy"),
        status_filter_ongoing: String::from("En cours"),
        status_filter_completed: String::from("Terminé"),
        // ... other site-specific settings
    }
}
```

### Key Configuration Fields
- `base_url`: Site base URL
- `description_selector`: CSS selector for manga description
- `date_format`: Date parsing format for chapters
- `status_filter_*`: Status strings in French
- `popular`/`trending`: Listing names
- `alt_ajax`: Use alternative AJAX endpoint
- Custom closures for `status` and `nsfw` parsing

## Adding New Madara Source

1. **Create source directory**: `sources/newsitename/`
2. **Copy template structure**:
   ```bash
   cp -r template/ sources/newsitename/
   ```
3. **Configure `src/lib.rs`**: Update `get_data()` with site-specific settings
4. **Update `res/source.json`**: Set source metadata (ID, name, URL)
5. **Configure `res/filters.json`**: Define search filters
6. **Add icon**: Place `Icon.png` in `res/`
7. **Test build**: `./build.sh newsitename`

## Common Issues

- **Date parsing**: Adjust `date_format` for site-specific date formats
- **Status mapping**: Update French status strings in filters
- **AJAX endpoints**: Toggle `alt_ajax` if search doesn't work
- **Selectors**: Modify CSS selectors for site structure changes