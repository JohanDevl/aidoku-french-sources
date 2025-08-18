# 📚 Aidoku French Sources

[![Sources](https://img.shields.io/badge/sources-12-blue.svg)](https://github.com/JohanDevl/aidoku-french-sources)
[![Active](https://img.shields.io/badge/active-8-green.svg)](https://github.com/JohanDevl/aidoku-french-sources)
[![Status](https://img.shields.io/badge/status-maintained-brightgreen.svg)](https://github.com/JohanDevl/aidoku-french-sources)

This repository hosts French manga/scan sources compatible with the [Aidoku](https://aidoku.app/) application. Aidoku is a free and open-source manga reader for iOS that allows reading manga from various sources.

## 🌐 Website

Check out our interactive website to browse available sources: **[johandevl.github.io/aidoku-french-sources](https://johandevl.github.io/aidoku-french-sources/)**

## 🚀 Quick Installation

### Method 1: Direct Link (Recommended)

Click this link from your iOS device with Aidoku installed:

**[➕ Add this source list](https://aidoku.app/add-source-list/?url=https://raw.githubusercontent.com/JohanDevl/aidoku-french-sources/sources/)**

### Method 2: Manual Installation

1. Open the **Aidoku** app
2. Go to **Settings** → **Source Lists**
3. Tap **Add Source List**
4. Paste this URL: `https://raw.githubusercontent.com/JohanDevl/aidoku-french-sources/sources/`
5. Tap **Add**

## 📖 Available Sources

|        Name        |                        URL                        |     Status     |    Type     | Description                   |
| :----------------: | :-----------------------------------------------: | :------------: | :---------: | :---------------------------- |
| **MangasOrigines** | [mangas-origines.fr](https://mangas-origines.fr/) | ✅ **Active**  |   Madara    | Large catalog of French manga |
| **MangaScantrad**  |  [manga-scantrad.io](https://manga-scantrad.io/)  | ✅ **Active**  |   Madara    | Quality French scanlations    |
|  **Astral Manga**  |    [astral-manga.fr](https://astral-manga.fr/)    | ✅ **Active**  |   Madara    | Popular manga in French       |
|   **LelscanFR**    |      [lelscanfr.com](https://lelscanfr.com/)      | ✅ **Active**  |   Custom    | Recent French scanlations     |
|  **PhenixScans**   |   [phenix-scans.com](https://phenix-scans.com/)   | ✅ **Active**  |   Custom    | French scanlation community   |
| **PoseidonScans**  |   [poseidonscans.com](https://poseidonscans.com/) | ✅ **Active**  |   Custom    | Next.js scanlation platform  |
|    **AnimeSama**   |       [anime-sama.fr](https://anime-sama.fr/)     | ✅ **Active**  |   Custom    | Anime/Manga hybrid platform  |
|   **SushiScans**   |       [sushiscan.fr](https://sushiscan.fr/)       | ✅ **Active**  | MangaStream | Various French scanlations    |
|   **SushiScan**    |      [sushiscan.net](https://sushiscan.net/)      | ❌ **Offline** | MangaStream | Chapter loading issues        |
| **ReaperScansFR**  |    [reaper-scans.fr](https://reaper-scans.fr/)    | ❌ **Offline** |   Madara    | Website temporarily down      |
|   **Manga Scan**   |   [mangascan-fr.com](https://mangascan-fr.com/)   | ❌ **Offline** |   MMRCMS    | Website temporarily down      |
|  **LegacyScans**   |   [legacy-scans.com](https://legacy-scans.com/)   | ❌ **Offline** |   Custom    | Website temporarily down      |

### Status Legend

- ✅ **Active**: Source is functional and updated
- ❌ **Offline**: Website inaccessible or source not working

## 🛠️ Technical Architecture

This project uses **Rust** and different templates to support various website technologies:

### Supported Templates

|    Template     |           Description           |              Compatible Sites               |
| :-------------: | :-----------------------------: | :-----------------------------------------: |
|   **Madara**    | Popular WordPress CMS for manga | MangasOrigines, MangaScantrad, Astral Manga |
| **MangaStream** |    Manga streaming framework    |            SushiScans, SushiScan            |
|   **MMRCMS**    |     Manga management system     |                 Manga Scan                  |
|   **Custom**    |      Custom implementation      |   LelscanFR, PhenixScans, PoseidonScans, AnimeSama, LegacyScans   |

### Project Structure

```
src/rust/
├── madara/          # Template for Madara sites
├── mangastream/     # Template for MangaStream sites
├── mmrcms/          # Template for MMRCMS sites
├── fr.lelscanfr/    # Custom source for LelscanFR
├── fr.phenixscans/  # Custom source for PhenixScans
├── fr.poseidonscans/ # Custom source for PoseidonScans
├── fr.animesama/    # Custom source for AnimeSama
└── fr.legacyscans/  # Custom source for LegacyScans
```

## 👨‍💻 For Developers

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Git](https://git-scm.com/)

### Local Installation

```bash
git clone https://github.com/JohanDevl/aidoku-french-sources.git
cd aidoku-french-sources
```

### Building a Source

```bash
# For a Madara source
cd src/rust/madara && ./build.sh

# For a custom source
cd src/rust/fr.lelscanfr && ./build.sh
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
