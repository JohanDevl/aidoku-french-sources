# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Development Commands

### Building Sources

**All sources (unified structure):**
```bash
# Build a single source
cd sources/fr.lelscanfr && aidoku package

# Build all active sources
for src in sources/*; do (cd "$src" && aidoku package); done

# Build offline sources (if needed)
for src in offline-sources/*; do (cd "$src" && aidoku package); done
```

**Prerequisites:**
- Install [aidoku-cli](https://github.com/Aidoku/aidoku-rs): `cargo install --git https://github.com/Aidoku/aidoku-rs aidoku-cli`
- Stable Rust toolchain with wasm32-unknown-unknown target

### Building with Rust toolchain
All sources use stable Rust toolchain with wasm32-unknown-unknown target:
```bash
cargo build --release --target wasm32-unknown-unknown
```

**Note:** For offline sources, always use the `--target wasm32-unknown-unknown` flag to prevent linking issues.

## High-level Architecture

### Project Structure
This is an Aidoku source repository containing French manga sources written in Rust that compile to WebAssembly (.wasm) and package as .aix files.

```
├── sources/           # Active sources (flat structure)
│   ├── fr.animesama/
│   ├── fr.fmteam/
│   ├── fr.lelmanga/
│   ├── fr.lelscanfr/
│   ├── fr.mangascantrad/
│   ├── fr.mangasorigines/
│   ├── fr.phenixscans/
│   ├── fr.poseidonscans/
│   └── fr.sushiscans/
├── offline-sources/   # Offline/inactive sources
│   ├── fr.astralmanga/
│   ├── fr.legacyscans/
│   ├── fr.mangascan/
│   ├── fr.reaperscansfr/
│   └── fr.sushiscan/
├── templates/         # Reusable templates (for offline sources only)
│   ├── madara/                 # WordPress Madara theme template
│   ├── mangastream/            # MangaStream framework template
│   └── mmrcms/                 # MMRCMS template
└── public/           # Website files
```

### Template System
The project uses centralized templates for offline sources only. Active sources use custom implementations.

**Templates (for offline sources only):**
- **templates/madara/** - WordPress Madara theme used by offline sources (fr.astralmanga, fr.legacyscans, fr.reaperscansfr)
- **templates/mangastream/** - MangaStream framework used by offline sources (fr.sushiscan)
- **templates/mmrcms/** - MMRCMS used by offline sources (fr.mangascan)

**Note:** Templates are self-contained and include all necessary types and utilities. No wrapper dependencies are required.

**Active sources** - All use custom implementations: AnimeSama, FMTeam, LelManga, LelscanFR, MangaScantrad, MangasOrigines, PhenixScans, PoseidonScans, SushiScans

### Source Configuration
Each source has a standardized structure:
- `res/source.json` - Metadata (id, name, version, URL)
- `res/filters.json` - Search/filter configuration
- `res/icon.png` - Source icon (lowercase filename)
- `src/lib.rs` - Main source implementation

### Key Implementation Details

**Offline sources (template-based):**
- Use templates as dependencies: `template_name_template = { path = "../../templates/template_name" }`
- Self-contained with no additional wrapper dependencies required
- Minimal implementation that configures the template
- Must be compiled with `--target wasm32-unknown-unknown` to prevent linking issues
- Located in `offline-sources/` directory

**Active sources (custom implementations):**
- Standalone implementations using modern Aidoku SDK
- Direct implementation of Aidoku trait functions: `get_manga_list`, `get_manga_details`, `get_chapter_list`, `get_page_list`
- Full control over parsing and data handling
- Located in `sources/` directory

### Deployment
- GitHub Actions builds all sources automatically
- Outputs .aix packages and website to `sources` branch (from main) or `sources-develop` branch (from develop)
- Website files are stored in `public/` directory
- Each deployment contains: website files, index.json, sources/ directory with .aix files

## Development Workflow

### Adding a New Source

1. **Create source directory:**
   ```bash
   mkdir sources/fr.newsite
   cd sources/fr.newsite
   ```

2. **Create basic structure:**
   ```bash
   mkdir -p res src
   ```

3. **Add source metadata:** Create `res/source.json`
   ```json
   {
     "info": {
       "id": "fr.newsite",
       "name": "NewSite",
       "version": 1,
       "url": "https://newsite.com/",
       "contentRating": 1,
       "languages": ["fr"]
     }
   }
   ```

4. **Add icon:** Place a 256x256 PNG icon as `res/icon.png`

5. **Choose implementation approach:**
   
   **For active sources (recommended):**
   ```toml
   # Cargo.toml
   [dependencies]
   aidoku = { git = "https://github.com/Aidoku/aidoku-rs/", features = ["json"] }
   serde = { version = "1.0", features = ["derive"] }
   chrono = "0.4"
   ```
   
   **For offline sources (template approach):**
   ```toml
   # Cargo.toml
   [dependencies]
   madara_template = { path = "../../templates/madara" }
   # OR
   mangastream_template = { path = "../../templates/mangastream" }
   # OR
   mmrcms_template = { path = "../../templates/mmrcms" }
   ```

6. **Implement source logic** in `src/lib.rs`

7. **Test locally:**
   ```bash
   aidoku package
   aidoku verify package.aix
   ```

### Testing and Validation

**Build and test specific source:**
```bash
cd sources/fr.newsource
aidoku package && aidoku verify package.aix
```

**Test all active sources:**
```bash
for src in sources/*; do
  echo "Testing $(basename "$src")..."
  (cd "$src" && aidoku package && aidoku verify package.aix)
done
```

**Test offline sources:**
```bash
for src in offline-sources/*; do
  echo "Testing $(basename "$src")..."
  (cd "$src" && cargo build --release --target wasm32-unknown-unknown && aidoku package && aidoku verify package.aix)
done
```

### Managing Offline Sources

**Moving a source to offline status:**
```bash
mv sources/fr.sourcename offline-sources/
```

**Reactivating an offline source:**
```bash
mv offline-sources/fr.sourcename sources/
```

### Template Development

**Note:** Template development only applies to offline sources in `offline-sources/`.

When modifying stable templates, test with all dependent offline sources:

**Test templates:**
```bash
# Build all templates first
for template in templates/*; do
  echo "Testing $(basename "$template")..."
  (cd "$template" && cargo build --release --target wasm32-unknown-unknown)
done

# Build all offline sources using templates
for src in offline-sources/*; do
  echo "Testing $(basename "$src")..."
  (cd "$src" && cargo build --release --target wasm32-unknown-unknown && aidoku package)
done
```

## Troubleshooting

### Common Build Issues

1. **Template not found:** Check `Cargo.toml` path to template
2. **Missing dependencies:** Run `cargo clean` and rebuild
3. **Icon issues:** Ensure icon is named `icon.png` (lowercase)
4. **Version mismatches:** Update aidoku dependency version
5. **Linking errors for offline sources:** Always use `--target wasm32-unknown-unknown` when building offline sources
6. **Panic handler errors:** Templates include required allocators and panic handlers for no_std environment

### Quick Fixes

```bash
# Clean all build artifacts
find sources offline-sources templates -name "target" -type d -exec rm -rf {} +
find sources offline-sources -name "*.aix" -delete

# Rebuild active sources
for src in sources/*; do (cd "$src" && cargo clean && aidoku package); done

# Rebuild offline sources (if needed)
for src in offline-sources/*; do (cd "$src" && cargo clean && cargo build --release --target wasm32-unknown-unknown && aidoku package); done
```

# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.