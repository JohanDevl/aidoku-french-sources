# 🗺️ Roadmap - French Sources Implementation

This document tracks the upcoming French manga sources to implement and work in progress sources for Aidoku French Sources.

## 📊 Current Status

- **Implemented sources**: 17 (10 active + 7 offline)
- **Remaining sources**: 5
- **Total available sources**: 22

## 🔄 Work in Progress

Currently no sources are being actively developed.

## 🆙 Upcoming Sources

### Standard Priority Sources

| Source Name      | URL                                          | Expected Type | Priority  | Notes                 |
| ---------------- | -------------------------------------------- | ------------- | --------- | --------------------- |
| **BananaScan**   | [harmony-scan.fr](https://harmony-scan.fr)   | Unknown       | 🟡 Medium | Community source      |
| **EdScanlation** | [edscanlation.fr](https://edscanlation.fr)   | Unknown       | 🟡 Medium | Scanlation team       |
| **MangasScans**  | [mangas-scans.com](https://mangas-scans.com) | Unknown       | 🟡 Medium | Manga scanning source |
| **RaijinScans**  | [raijinscan.co](https://raijinscan.co)       | Unknown       | 🟡 Medium | Scanlation group      |
| **RimuScans**    | [rimuscans.com](https://rimuscans.com)       | Unknown       | 🟡 Medium | Scanlation team       |

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

_Last updated: 2025-10-07_
_Total sources tracked: 29 upcoming_
