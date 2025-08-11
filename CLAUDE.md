# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Environment

- **Language**: Rust with `nightly` toolchain
- **Target**: WebAssembly (WASM32)
- **Development tools**: Nix flake available with `aidoku-cli`, `rustup`, and `mitmproxy`
- **Package format**: `.aix` files (Aidoku source packages)

### Setup Commands

```bash
# Use Nix development environment (recommended)
nix develop

# Or manually install dependencies
rustup install nightly
rustup default nightly
```

## Build Commands

### Template-based Sources (Madara, MangaStream, MMRCMS)

```bash
# Build specific source (e.g., for Madara)
cd src/rust/madara && ./build.sh sourcename

# Build all sources in template
cd src/rust/madara && ./build.sh -a
```

### Custom Sources

```bash
# Build individual custom source
cd src/rust/fr.lelscanfr && ./build.sh

# Or using cargo directly
cargo +nightly build --release
```

### Package Verification

```bash
# Verify package contents
aidoku package package.aix
aidoku verify package.aix
```

## Project Architecture

This repository contains **Aidoku sources** for French manga sites, built using **Rust** targeting **WebAssembly**.

### Source Organization

- **Template-based sources**: Use shared templates for common site frameworks
  - `madara/`: WordPress Madara CMS sites
  - `mangastream/`: MangaStream framework sites  
  - `mmrcms/`: MMRCMS framework sites
- **Custom sources**: Individual implementations for unique sites
  - `fr.lelscanfr/`: LelscanFR custom implementation
  - `fr.phenixscans/`: PhenixScans custom implementation
  - `fr.legacyscans/`: LegacyScans custom implementation

### Template Structure

Templates contain:
- `template/src/`: Core template logic with configurable `MadaraSiteData` struct
- `sources/[site]/`: Site-specific configurations and overrides
- Shared `Cargo.toml` workspace for all sources in template

### Build Process

1. **Compile**: Rust ’ WASM using `cargo +nightly build --release`
2. **Package**: Create `.aix` file with WASM binary + resources (`Icon.png`, `source.json`, `filters.json`)
3. **Distribute**: Package uploaded to source repository

### Code Formatting

- Uses `rustfmt.toml` with hard tabs and comment wrapping
- Run `cargo fmt` before committing

## Key Files

- `src/rust/[source]/res/source.json`: Source metadata and configuration
- `src/rust/[source]/res/filters.json`: Search filters definition
- `src/rust/[source]/src/lib.rs`: Main source implementation
- Template files provide reusable logic for common site types