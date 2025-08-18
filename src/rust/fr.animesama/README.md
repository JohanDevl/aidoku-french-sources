# AnimeSama Source

Custom Aidoku source for the French manga site [anime-sama.fr](https://anime-sama.fr).

## Overview

- **Site**: anime-sama.fr
- **Type**: Custom (Advanced Hybrid)
- **Status**: ✅ Active
- **Language**: French
- **NSFW**: Safe content only
- **Package Size**: ~45KB

## Features

### Listings
- **Dernières Sorties**: Latest manga releases from homepage
- **Populaire**: Popular manga from catalog with pagination

### Advanced Parsing
- ✅ JavaScript command interpretation (creerListe, newSP, finirListe)
- ✅ Multi-layer chapter discovery (HTML → API → Episodes.js → Fallback)
- ✅ Smart CDN URL construction with proper encoding
- ✅ Special chapter handling (decimal numbers, custom titles)
- ✅ One Piece special case support (scan_noir-et-blanc)

### Search & Browse
- ✅ Title search with URL encoding
- ✅ Genre filtering with multi-select
- ✅ Catalog browsing with pagination

## Technical Highlights

### Hybrid Architecture
This source represents one of the most sophisticated implementations in the repository:

- **HTML scraping** for manga listings and basic info
- **JavaScript parsing** for dynamic chapter mapping
- **API integration** for accurate chapter/page counts
- **CDN optimization** for fast image loading

### JavaScript Command System
Parses embedded JavaScript commands to build accurate chapter lists:

```javascript
creerListe(1, 100);     // Chapters 1-100
newSP("One Shot");      // Special chapter
newSP(19.5);            // Decimal chapter
finirListe(27);         // Continue from 27+
```

### Smart URL Construction
```
https://anime-sama.fr/s2/scans/{encoded_title}/{chapter_index}/{page}.jpg
```

## Build

```bash
cd src/rust/fr.animesama
./build.sh
```

## Architecture

```
fr.animesama/
├── src/
│   ├── lib.rs           # Main implementation
│   ├── parser.rs        # Advanced parsing logic
│   ├── parser_new.rs    # Simplified alternative
│   └── helper.rs        # URL encoding utilities
├── res/
│   ├── source.json      # Source metadata
│   ├── filters.json     # Search filters
│   └── Icon.png         # Source icon
└── CLAUDE.md           # Detailed technical documentation
```

## Special Features

- **Anti-bot measures**: Complete browser header simulation
- **Error resilience**: Multiple fallback strategies
- **Debug capabilities**: Extensive diagnostic information
- **Performance optimized**: Parallel API calls and efficient parsing

This source showcases advanced Aidoku development techniques and serves as a reference implementation for complex manga sites requiring sophisticated parsing strategies.