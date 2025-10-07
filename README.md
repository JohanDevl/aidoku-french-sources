# 📚 Aidoku French Sources

[![Sources](https://img.shields.io/badge/sources-17-blue.svg)](https://github.com/JohanDevl/aidoku-french-sources)
[![Active](https://img.shields.io/badge/active-10-green.svg)](https://github.com/JohanDevl/aidoku-french-sources)
[![Status](https://img.shields.io/badge/status-maintained-brightgreen.svg)](https://github.com/JohanDevl/aidoku-french-sources)

This repository hosts French manga/scan sources compatible with the [Aidoku](https://aidoku.app/) application. Aidoku is a free and open-source manga reader for iOS that allows reading manga from various sources.

## 🌐 Website

Check out our interactive website to browse available sources: **[johandevl.github.io/aidoku-french-sources](https://johandevl.github.io/aidoku-french-sources/)**

## 🚀 Quick Installation

### Method 1: Direct Link (Recommended)

Click this link from your iOS device with Aidoku installed:

**[➕ Add this source list](https://aidoku.app/add-source-list/?url=https://johandev.com/aidoku-french-sources/index.min.json)**

### Method 2: Manual Installation

1. Open the **Aidoku** app
2. Go to **Settings** → **Source Lists**
3. Tap **Add Source List**
4. Paste this URL: `https://johandev.com/aidoku-french-sources/index.min.json`
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
|   **AnimeSama**    |      [anime-sama.fr](https://anime-sama.fr/)      | ✅ **Active**  |    Custom     | Anime/Manga hybrid platform   |
|     **FMTeam**     |        [fmteam.fr](https://fmteam.fr/)        | ✅ **Active**  |    Custom     | French scanlation team        |
|    **LelManga**    |     [lelmanga.com](https://www.lelmanga.com/)     | ✅ **Active**  | MangaThemesia | French manga catalog          |
|   **SushiScans**   |       [sushiscan.fr](https://sushiscan.fr/)       | ✅ **Active**  |  MangaStream  | Various French scanlations    |
| **Starbound Scans** | [starboundscans.com](https://starboundscans.com/) | ✅ **Active**  |    Custom     | French manhwa scanlations     |
|     **JapScan**     |       [japscan.si](https://www.japscan.si/)       | ❌ **Offline** |    Custom     | Dynamic JS/Shadow DOM incompatible |
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
|    **Madara**     | Popular WordPress CMS for manga |                  MangasOrigines, MangaScantrad                  |
| **MangaThemesia** | WordPress theme for manga sites |                           LelManga                            |
|  **MangaStream**  |    Manga streaming framework    |                     SushiScans, SushiScan                     |
|    **MMRCMS**     |     Manga management system     |                          Manga Scan                           |
|    **Custom**     |      Custom implementation      | LelscanFR, PhenixScans, PoseidonScans, AnimeSama, FMTeam, Starbound Scans, JapScan, CrunchyScan, LegacyScans |

### Project Structure

```
sources/
├── fr.animesama/    # Custom source for AnimeSama
├── fr.fmteam/       # Custom source for FMTeam
├── fr.lelmanga/     # MangaThemesia source for LelManga
├── fr.lelscanfr/    # Custom source for LelscanFR
├── fr.mangascantrad/ # Madara source for MangaScantrad
├── fr.mangasorigines/ # Madara source for MangasOrigines
├── fr.phenixscans/  # Custom source for PhenixScans
├── fr.poseidonscans/ # Custom source for PoseidonScans
├── fr.starboundscans/ # Custom source for Starbound Scans
└── fr.sushiscans/   # MangaStream source for SushiScans
```

## 👨‍💻 For Developers

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version with nightly toolchain)
- [Git](https://git-scm.com/)
- [Aidoku CLI](https://github.com/Aidoku/aidoku-rs): `cargo install --git https://github.com/Aidoku/aidoku-rs aidoku-cli`

### Local Installation

```bash
git clone https://github.com/JohanDevl/aidoku-french-sources.git
cd aidoku-french-sources

# Install nightly Rust with WASM target
rustup install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

### Building a Source

```bash
# For template-based sources (Madara, MangaStream, MMRCMS)
cd src/rust/madara/sources/lelmanga && RUSTUP_TOOLCHAIN=nightly aidoku package
cd src/rust/mangastream/sources/sushiscans && RUSTUP_TOOLCHAIN=nightly aidoku package

# For custom sources
cd src/rust/fr.lelscanfr && RUSTUP_TOOLCHAIN=nightly aidoku package
cd src/rust/fr.phenixscans && RUSTUP_TOOLCHAIN=nightly aidoku package
```

### Adding a New Source

1. **Identify the site type** (Madara, MangaStream, MMRCMS, or Custom)
2. **Create the configuration** in the appropriate folder
3. **Test the source** locally
4. **Submit a Pull Request**

For more details, check the corresponding template in `src/rust/[template]/template/`.

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
