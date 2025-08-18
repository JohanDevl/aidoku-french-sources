# SushiScans Legacy Source

MangaStream template-based source for [sushiscan.fr](https://sushiscan.fr) (Legacy domain).

## Overview

- **Site**: sushiscan.fr (Legacy domain)
- **Template**: MangaStream (Standard)
- **Status**: ✅ Active
- **Language**: French

## Features

### Standard MangaStream Capabilities
- ✅ Manga listings and browsing
- ✅ Search functionality
- ✅ Genre and status filtering
- ✅ Chapter reading support
- ✅ Standard MangaStream parsing

## Build

```bash
cd src/rust/mangastream
./build.sh sushiscans
```

## Configuration

Uses standard MangaStream template configuration for the legacy SushiScan domain.

### Key Differences from v6
- **Standard Parsing**: Uses traditional MangaStream selectors
- **No Alt Pages**: Standard HTML structure parsing
- **Legacy Domain**: sushiscan.fr instead of sushiscan.net

This source maintains compatibility with the legacy SushiScan domain using standard MangaStream template features.