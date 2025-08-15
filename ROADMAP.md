# 🗺️ Roadmap - French Sources Implementation

This document tracks the upcoming French manga sources to implement and work in progress sources for Aidoku French Sources.

## 📊 Current Status

- **Implemented sources**: 10 (6 active + 4 offline)
- **Remaining sources**: 36
- **Total available sources**: 46

## 🔄 Work in Progress

Currently no sources are being actively developed.

## 🆙 Upcoming Sources

### Standard Priority Sources

| Source Name         | URL                | Expected Type | Priority  | Notes                      |
| ------------------- | ------------------ | ------------- | --------- | -------------------------- |
| **JapScan**         | japscan.si         | Custom        | 🔴 High   | Major French manga source  |
| **Manga-Kawaii**    | mangakawaii.io     | Unknown       | 🔴 High   | Popular community site     |
| **FlameScansFR**    | legacy-scans.com   | Unknown       | 🔴 High   | Active scanlation group    |
| **Scan-VF**         | scan-vf.net        | Unknown       | 🔴 High   | Well-known French site     |
| **AnimeSama**       | anime-sama.fr      | Custom        | 🟡 Medium | Anime/Manga hybrid site    |
| **AnteikuScan**     | anteikuscan.fr     | Unknown       | 🟡 Medium | Scanlation group           |
| **BananaScan**      | harmony-scan.fr    | Unknown       | 🟡 Medium | Community source           |
| **EdScanlation**    | edscanlation.fr    | Unknown       | 🟡 Medium | Scanlation team            |
| **EnligneManga**    | enlignemanga.com   | Unknown       | 🟡 Medium | Online manga platform      |
| **EpsilonScan**     | epsilonscan.to     | Unknown       | 🟡 Medium | Scanlation group           |
| **FMTeam**          | fmteam.fr          | Unknown       | 🟡 Medium | French scanlation team     |
| **FrManga**         | frmanga.com        | Unknown       | 🟡 Medium | French manga source        |
| **FuryoSquad**      | furyosociety.com   | Unknown       | 🟡 Medium | Scanlation group           |
| **InovaScanManga**  | inovascanmanga.com | Unknown       | 🟡 Medium | Scan community             |
| **LelManga**        | lelmanga.com       | Unknown       | 🟡 Medium | Related to LelscanFR       |
| **MangaHubFR**      | mangahub.fr        | Unknown       | 🟡 Medium | French manga hub           |
| **MangasScans**     | mangas-scans.com   | Unknown       | 🟡 Medium | Manga scanning source      |
| **PantheonScan**    | pantheon-scan.com  | Unknown       | 🟡 Medium | Scanlation group           |
| **PerfScan**        | perf-scan.net      | Unknown       | 🟡 Medium | Quality-focused scans      |
| **PoseidonScans**   | poseidonscans.com  | Unknown       | 🟡 Medium | Scanlation team            |
| **RaijinScans**     | raijinscan.co      | Unknown       | 🟡 Medium | Scanlation group           |
| **RimuScans**       | rimuscans.com      | Unknown       | 🟡 Medium | Scanlation team            |
| **RoyalManga**      | royalmanga.com     | Unknown       | 🟡 Medium | Premium manga source       |
| **ScanTradUnion**   | scantrad-union.com | Unknown       | 🟡 Medium | Scanlation union           |
| **ScanVFOrg**       | scanvf.org         | Unknown       | 🟡 Medium | VF scanning organization   |
| **SoftEpsilonScan** | epsilonsoft.to     | Unknown       | 🟡 Medium | Related to EpsilonScan     |
| **StarboundScans**  | starboundscans.com | Unknown       | 🟡 Medium | Scanlation group           |
| **ToonFR**          | toonfr.com         | Unknown       | 🟡 Medium | French toon/webtoon source |

## 🏗️ Implementation Guidelines

### Before Starting a Source

1. **Research the site architecture**:

   - Check if it uses a known template (Madara, MangaStream, MMRCMS)
   - Analyze the HTML structure and API endpoints
   - Test the site availability and stability

2. **Update this roadmap**:

   - Move the source from "Upcoming" to "Work in Progress"
   - Add your name and start date
   - Include any specific technical notes

3. **Check existing implementations**:
   - Look for similar sites already implemented
   - Reuse templates when possible

### Implementation Process

1. **Site Analysis** - Understand the site structure
2. **Template Selection** - Choose appropriate template or custom implementation
3. **Configuration** - Create site-specific configuration files
4. **Testing** - Verify functionality with real data
5. **Documentation** - Update README and this roadmap
6. **Package Build** - Generate .aix package and test

### After Implementation

1. **Update README.md** - Add the new source to the main list
2. **Update this roadmap** - Remove from upcoming, update counters
3. **Test thoroughly** - Ensure the source works correctly
4. **Update CLAUDE.md** - Document any new patterns or learnings

## 📝 Notes

- Sources marked as "Unknown" type need investigation to determine their framework
- Priority levels are suggestions based on popularity and community requests
- Some sources might be offline or have changed URLs since the original reference list
- Adult content sources are marked separately and have lower priority

## 🤝 Contributing

When working on a source:

1. Comment on the relevant issue or create one
2. Update this roadmap to show work in progress
3. Follow the implementation guidelines above
4. Test thoroughly before submitting PR

---

_Last updated: 2025-08-15_
_Total sources tracked: 36 upcoming_
