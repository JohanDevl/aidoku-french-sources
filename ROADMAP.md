# 🗺️ Roadmap - French Sources Implementation

This document tracks the upcoming French manga sources to implement and work in progress sources for Aidoku French Sources.

## 📊 Current Status

- **Implemented sources**: 10 (6 active + 4 offline)
- **Remaining sources**: 36
- **Total available sources**: 46

## 🔄 Work in Progress

Currently no sources are being actively developed.

## 🆙 Upcoming Sources

### High Priority Sources

These are popular sources that should be prioritized for implementation:

| Source Name | URL | Expected Type | Priority | Notes |
|-------------|-----|---------------|----------|-------|
| **JapScan** | japscan.co | Custom | 🔴 High | Major French manga source |
| **Manga-Kawaii** | manga-kawaii.com | Unknown | 🔴 High | Popular community site |
| **FlameScansFR** | flamescansfr.com | Unknown | 🔴 High | Active scanlation group |
| **Scan-VF** | scan-vf.net | Unknown | 🔴 High | Well-known French site |

### Standard Priority Sources

| Source Name | Expected Type | Priority | Notes |
|-------------|---------------|----------|-------|
| **AnimeSama** | Custom | 🟡 Medium | Anime/Manga hybrid site |
| **AnteikuScan** | Unknown | 🟡 Medium | Scanlation group |
| **BananaScan** | Unknown | 🟡 Medium | Community source |
| **EdScanlation** | Unknown | 🟡 Medium | Scanlation team |
| **EnligneManga** | Unknown | 🟡 Medium | Online manga platform |
| **EpsilonScan** | Unknown | 🟡 Medium | Scanlation group |
| **FMTeam** | Unknown | 🟡 Medium | French scanlation team |
| **FrManga** | Unknown | 🟡 Medium | French manga source |
| **FuryoSquad** | Unknown | 🟡 Medium | Scanlation group |
| **InovaScanManga** | Unknown | 🟡 Medium | Scan community |
| **LelManga** | Unknown | 🟡 Medium | Related to LelscanFR |
| **MangaHubFR** | Unknown | 🟡 Medium | French manga hub |
| **MangasScans** | Unknown | 🟡 Medium | Manga scanning source |
| **PantheonScan** | Unknown | 🟡 Medium | Scanlation group |
| **PerfScan** | Unknown | 🟡 Medium | Quality-focused scans |
| **PoseidonScans** | Unknown | 🟡 Medium | Scanlation team |
| **RaijinScans** | Unknown | 🟡 Medium | Scanlation group |
| **RimuScans** | Unknown | 🟡 Medium | Scanlation team |
| **RoyalManga** | Unknown | 🟡 Medium | Premium manga source |
| **ScanTradUnion** | Unknown | 🟡 Medium | Scanlation union |
| **ScanVFOrg** | Unknown | 🟡 Medium | VF scanning organization |
| **SoftEpsilonScan** | Unknown | 🟡 Medium | Related to EpsilonScan |
| **StarboundScans** | Unknown | 🟡 Medium | Scanlation group |
| **ToonFR** | Unknown | 🟡 Medium | French toon/webtoon source |

### Specialized Sources

| Source Name | Type | Priority | Notes |
|-------------|------|----------|-------|
| **AraLosBD** | BD/Comics | 🟢 Low | BD-focused content |
| **BluesOlo** | Unknown | 🟢 Low | Niche content |
| **HentaiOrigines** | Adult | 🟢 Low | Adult content source |
| **HentaiScantrad** | Adult | 🟢 Low | Adult scanlations |
| **HentaiZone** | Adult | 🟢 Low | Adult manga zone |
| **HistoireHentai** | Adult | 🟢 Low | Adult story content |
| **LunarScansHentai** | Adult | 🟢 Low | Adult lunar scans |
| **ScanHentaiMenu** | Adult | 🟢 Low | Adult scan menu |
| **YaoiScan** | Adult | 🟢 Low | Yaoi-specific content |

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

*Last updated: 2025-01-14*
*Total sources tracked: 36 upcoming*