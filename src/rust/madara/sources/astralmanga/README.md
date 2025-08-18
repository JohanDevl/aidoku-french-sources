# AstralManga Source

Madara template-based source for [astral-manga.fr](https://astral-manga.fr).

## Overview

- **Site**: astral-manga.fr
- **Template**: Madara CMS
- **Status**: ✅ Active
- **Language**: French
- **Version**: 5
- **NSFW**: Safe content only

## Features

### Listings
- **Tendance**: Trending manga
- **Populaire**: Popular manga

### Madara Capabilities
- ✅ Advanced search with filters
- ✅ Genre filtering (Action, Adventure, Comedy, etc.)
- ✅ Status filtering (En cours, Terminé)
- ✅ Type filtering (Manga, Manhwa, Manhua)
- ✅ Multiple sorting options
- ✅ AJAX-based pagination

## Build

```bash
cd src/rust/madara
./build.sh astralmanga
```

## Configuration

Uses standard Madara template with site-specific settings:

```rust
fn get_data() -> template::MadaraSiteData {
    template::MadaraSiteData {
        base_url: String::from("https://astral-manga.fr"),
        lang: String::from("fr"),
        description_selector: String::from("div.manga-excerpt p"),
        date_format: String::from("dd/MM/yyyy"),
        status_filter_ongoing: String::from("En cours"),
        status_filter_completed: String::from("Terminé"),
        popular: String::from("Populaire"),
        trending: String::from("Tendance"),
        ..Default::default()
    }
}
```

This source provides reliable access to AstralManga's French manga collection through the proven Madara template architecture.