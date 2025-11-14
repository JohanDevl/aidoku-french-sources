# 📚 Aidoku French Sources

[![Sources](https://img.shields.io/badge/sources-20-blue.svg)](https://github.com/JohanDevl/aidoku-french-sources)
[![Active](https://img.shields.io/badge/active-12-green.svg)](https://github.com/JohanDevl/aidoku-french-sources)
[![Status](https://img.shields.io/badge/status-maintained-brightgreen.svg)](https://github.com/JohanDevl/aidoku-french-sources)

This repository hosts French manga/scan sources compatible with the [Aidoku](https://aidoku.app/) application. Aidoku is a free and open-source manga reader for iOS that allows reading manga from various sources.

## 🌐 Website

Check out our interactive website to browse available sources: **[aidoku-french-sources.johandev.com](https://aidoku-french-sources.johandev.com/)**

## 🚀 Quick Installation

### Method 1: Direct Link (Recommended)

Click this link from your iOS device with Aidoku installed:

**[➕ Add this source list](https://aidoku.app/add-source-list/?url=https://aidoku-french-sources.johandev.com/index.min.json)**

### Method 2: Manual Installation

1. Open the **Aidoku** app
2. Go to **Settings** → **Source Lists**
3. Tap **Add Source List**
4. Paste this URL: `https://aidoku-french-sources.johandev.com/index.min.json`
5. Tap **Add**

### Method 3: Development Branch (Latest)

To test the latest development sources and features:

**[🧪 Add development sources](https://aidoku.app/add-source-list/?url=https://raw.githubusercontent.com/JohanDevl/aidoku-french-sources/sources-develop/index.min.json)**

Or manually add: `https://raw.githubusercontent.com/JohanDevl/aidoku-french-sources/sources-develop/index.min.json`

> ⚠️ **Note**: Development sources may be unstable and contain experimental features.

## 📖 Available Sources

|        Name        |                        URL                        |     Status     |     Type      | Description                   |
| :----------------: | :-----------------------------------------------: | :------------: | :-----------: | :---------------------------- |
| **MangasOrigines** | [mangas-origines.fr](https://mangas-origines.fr/) | ✅ **Active**  |    Madara     | Large catalog of French manga |
| **MangaScantrad**  |  [manga-scantrad.io](https://manga-scantrad.io/)  | ✅ **Active**  |    Madara     | Quality French scanlations    |
|  **Astral Manga**  |    [astral-manga.fr](https://astral-manga.fr/)    | ❌ **Offline** |    Madara     | Website temporarily down      |
|   **LelscanFR**    |      [lelscanfr.com](https://lelscanfr.com/)      | ✅ **Active**  |    Custom     | Recent French scanlations     |
|  **PhenixScans**   |   [phenix-scans.com](https://phenix-scans.com/)   | ✅ **Active**  |    Custom     | French scanlation community   |
| **PoseidonScans**  |  [poseidonscans.com](https://poseidonscans.com/)  | ✅ **Active**  |    Custom     | Next.js scanlation platform   |
|  **RaijinScans**   |    [raijinscan.co](https://raijinscan.co/)    | ✅ **Active**  |    Custom     | WordPress Madara-based scanlations |
|   **RimuScans**    |      [rimuscans.com](https://rimuscans.com/)      | ✅ **Active**  |    Custom     | French manga scanlation platform  |
|   **AnimeSama**    |      [anime-sama.fr](https://anime-sama.fr/)      | ✅ **Active**  |    Custom     | Anime/Manga hybrid platform   |
|     **FMTeam**     |        [fmteam.fr](https://fmteam.fr/)        | ✅ **Active**  |    Custom     | French scanlation team        |
|    **LelManga**    |     [lelmanga.com](https://www.lelmanga.com/)     | ✅ **Active**  | MangaThemesia | French manga catalog          |
|  **MangasScans**   |   [mangas-scans.com](https://mangas-scans.com/)   | ✅ **Active**  | MangaThemesia | French manga and manhwa       |
|   **SushiScans**   |       [sushiscan.fr](https://sushiscan.fr/)       | ✅ **Active**  |  MangaStream  | Various French scanlations    |
|     **JapScan**     |       [japscan.si](https://www.japscan.si/)       | ❌ **Offline** |    Custom     | Dynamic JS/Shadow DOM incompatible |
| **Starbound Scans** | [starboundscans.com](https://starboundscans.com/) | ❌ **Offline** |    Custom     | Merged with Poseidon Scans    |
|   **CrunchyScan**   |     [crunchyscan.fr](https://crunchyscan.fr/)     | ❌ **Offline** |    Custom     | Cloudflare interactive challenge |
|   **SushiScan**    |      [sushiscan.net](https://sushiscan.net/)      | ❌ **Offline** |  MangaStream  | Chapter loading issues        |
| **ReaperScansFR**  |    [reaper-scans.fr](https://reaper-scans.fr/)    | ❌ **Offline** |    Madara     | Website temporarily down      |
|   **Manga Scan**   |   [mangascan-fr.com](https://mangascan-fr.com/)   | ❌ **Offline** |    MMRCMS     | Website temporarily down      |
|  **LegacyScans**   |   [legacy-scans.com](https://legacy-scans.com/)   | ❌ **Offline** |    Custom     | Website temporarily down      |

### Status Legend

- ✅ **Active**: Source is functional and updated
- ❌ **Offline**: Website inaccessible or source not working

## 🛠️ Technical Architecture

This project uses **Rust** and different templates to support various website technologies:

### Supported Templates

|     Template      |           Description           |                       Compatible Sites                        |
| :---------------: | :-----------------------------: | :-----------------------------------------------------------: |
|    **Madara**     | Popular WordPress CMS for manga |            MangasOrigines, MangaScantrad            |
| **MangaThemesia** | WordPress theme for manga sites |                           LelManga                            |
|  **MangaStream**  |    Manga streaming framework    |                     SushiScans, SushiScan                     |
|    **MMRCMS**     |     Manga management system     |                          Manga Scan                           |
|    **Custom**     |      Custom implementation      | LelscanFR, PhenixScans, PoseidonScans, RaijinScans, RimuScans, AnimeSama, FMTeam, JapScan, CrunchyScan, LegacyScans, Starbound Scans |

### Project Structure

```
aidoku-french-sources/
├── sources/              # Active sources (12 sources)
│   ├── fr.animesama/
│   ├── fr.fmteam/
│   ├── fr.lelmanga/
│   ├── fr.lelscanfr/
│   ├── fr.mangascantrad/
│   ├── fr.mangasorigines/
│   ├── fr.mangasscans/
│   ├── fr.phenixscans/
│   ├── fr.poseidonscans/
│   ├── fr.raijinscans/
│   ├── fr.rimuscans/
│   └── fr.sushiscans/
├── offline-sources/      # Offline sources (8 sources)
│   ├── fr.astralmanga/
│   ├── fr.crunchyscan/
│   ├── fr.japscan/
│   ├── fr.legacyscans/
│   ├── fr.mangascan/
│   ├── fr.reaperscansfr/
│   ├── fr.starboundscans/
│   └── fr.sushiscan/
├── templates/            # Reusable templates (deprecated)
├── public/               # Website files
├── README.md
└── ROADMAP.md
```

## 👨‍💻 For Developers

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Git](https://git-scm.com/)
- [Aidoku CLI](https://github.com/Aidoku/aidoku-rs): `cargo install --git https://github.com/Aidoku/aidoku-rs aidoku-cli`

### Local Installation

```bash
# Clone the repository
git clone https://github.com/JohanDevl/aidoku-french-sources.git
cd aidoku-french-sources

# Add WASM target (if not already installed)
rustup target add wasm32-unknown-unknown
```

### Building a Source

All sources are built the same way using the Aidoku CLI:

```bash
# Navigate to the source directory
cd sources/fr.animesama

# Build the source package
aidoku package

# Verify the package
aidoku verify package.aix
```

#### Examples for Different Sources

```bash
# Custom sources
cd sources/fr.lelscanfr && aidoku package
cd sources/fr.phenixscans && aidoku package

# Template-based sources
cd sources/fr.mangascantrad && aidoku package
cd sources/fr.sushiscans && aidoku package
```

#### Building All Active Sources

```bash
# Build all active sources
for source in sources/*; do
  echo "Building $(basename "$source")..."
  (cd "$source" && aidoku package)
done
```

#### Note on Offline Sources

Sources in `offline-sources/` are **not built** by default. These are:
- Sources with technical incompatibilities (e.g., JapScan requires JavaScript)
- Sources with Cloudflare challenges (e.g., CrunchyScan)
- Temporarily offline websites

These sources are kept for:
- Documentation purposes
- Future reactivation if sites change
- Reference implementations

### Adding a New Source

1. **Create source directory**
   ```bash
   mkdir sources/fr.newsource
   cd sources/fr.newsource
   mkdir -p res src
   ```

2. **Add required files**
   - `res/source.json` - Source metadata
   - `res/icon.png` - 128x128 opaque PNG icon
   - `res/filters.json` - Search filters (optional)
   - `src/lib.rs` - Main implementation
   - `Cargo.toml` - Rust dependencies

3. **Implement the source**
   - Implement `Source` trait
   - Add parsers for manga list, details, chapters, pages
   - Test thoroughly

4. **Build and verify**
   ```bash
   aidoku package
   aidoku verify package.aix
   ```

5. **Submit a Pull Request**
   - Ensure all tests pass
   - Update README.md if needed
   - Follow commit message conventions

## 🤝 Contributing

Contributions are welcome! Here's how to participate:

1. 🍴 **Fork** this repository
2. 🔧 **Create** your feature branch (`git checkout -b feature/new-source`)
3. 💻 **Commit** your changes (`git commit -m 'Add new source'`)
4. 📤 **Push** to the branch (`git push origin feature/new-source`)
5. 🔄 **Open** a Pull Request

### Contribution Guidelines

- Follow the existing project structure
- Test your source before submitting
- Update documentation if necessary
- Use descriptive commit messages in English

## ❓ Frequently Asked Questions (FAQ)

### Q: How do I report an issue with a source?

A: Open an [issue](https://github.com/JohanDevl/aidoku-french-sources/issues) describing the problem encountered.

### Q: Why are some sources marked as "Offline"?

A: Websites may be temporarily inaccessible, under maintenance, or have changed their structure.

### Q: Can I request adding a new source?

A: Absolutely! Open an issue with the name and URL of the site you'd like to see added.

### Q: Are sources automatically updated?

A: Sources are updated regularly, but some changes may require manual intervention.

## 📜 License

This project is distributed under the MIT License. See the [LICENSE](LICENSE) file for more details.

## 🙏 Acknowledgments

- [Aidoku](https://aidoku.app/) for the application
- [Moomooo95/aidoku-french-sources](https://github.com/Moomooo95/aidoku-french-sources) for the original project this is based on
- The French scanlation community for their work
- All contributors to this project

---

<div align="center">
Made with ❤️ for the French manga community
</div>
