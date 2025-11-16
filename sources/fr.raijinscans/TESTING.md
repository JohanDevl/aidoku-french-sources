# RaijinScans Date Parsing Tests

## Test Cases for `parse_relative_date()`

This document provides manual test cases for the date parsing function based on actual RaijinScans HTML.

### Supported Formats

| Input | Expected Behavior | Notes |
|-------|-------------------|-------|
| `il y a 13h` | 13 hours ago | Abbreviated hours |
| `il y a 2 heures` | 2 hours ago | Full French word |
| `il y a 1j` | 1 day ago | Abbreviated days |
| `il y a 2j` | 2 days ago | Abbreviated days |
| `il y a 3 jours` | 3 days ago | Full French word |
| `il y a 1 semaine` | 1 week ago | Full French word |
| `il y a 2m` | 2 months ago | Abbreviated months ⚠️ |
| `il y a 5m` | 5 months ago | Abbreviated months ⚠️ |
| `il y a 1 mois` | 1 month ago | Full French word |
| `il y a 1 an` | 1 year ago | Singular year |
| `il y a 2 ans` | 2 years ago | Plural years |
| `aujourd'hui` | Today (now) | Special case |
| `hier` | Yesterday | Special case |
| `il y a 30min` | 30 minutes ago | Minutes (explicit) |

### Important Notes

⚠️ **Unit Ambiguity Resolution:**
- `"m"` alone = **months** (based on actual RaijinScans usage: "2m", "5m", "8m")
- `"min"` = **minutes** (to avoid confusion with "mois")
- This is correct because manga chapters are typically published monthly, not minutely

### Implementation Details

**Parsing Logic:**
1. Remove "il y a" prefix
2. Split into number and unit (two paths):
   - **Path 1:** Separated (e.g., "1 mois" → num=1, unit="mois")
   - **Path 2:** Combined (e.g., "13h" → num=13, unit="h")

**Unit Detection Order:**
1. Minutes (`min`, ` min`) - Most specific first
2. Hours (`h`, `heure`, `hour`)
3. Days (`j`, `jour`, `day`)
4. Weeks (`semaine`, `week`)
5. Months (`m`, `mois`, `month`) - After minutes to avoid conflicts
6. Years (`an`, `ans`, `year`) - Exact match to avoid false positives

**Timestamp Calculation:**
```rust
current_date() - offset  // Returns absolute Unix timestamp
```

**Time Constants:**
- 1 minute = 60 seconds
- 1 hour = 3,600 seconds
- 1 day = 86,400 seconds
- 1 week = 7 days
- **1 month = 30 days (approximation)**
- **1 year = 365 days (approximation)**

⚠️ **Known Limitation:** Months and years use fixed-day approximations:
- Month calculation assumes 30 days (not actual calendar months)
- Year calculation assumes 365 days (ignores leap years)
- This is acceptable for relative dates in manga chapter uploads
- Maximum error: ~2 days for "1 month ago", ~1 day for "1 year ago"

### Manual Testing Procedure

To manually test the date parsing:

1. **Build the source:**
   ```bash
   cd sources/fr.raijinscans
   aidoku package
   ```

2. **Test with real manga:**
   - Install the source in Aidoku app
   - Navigate to a manga with many chapters
   - Verify dates display correctly (not in 1969!)
   - Check premium chapters show dates

3. **Expected Results:**
   - Recent chapters: "13 hours ago", "2 days ago"
   - Older chapters: "2 months ago", "1 year ago"
   - All dates should be reasonable (not future dates, not 1969)

### Known Edge Cases

| Input | Result | Reason |
|-------|--------|--------|
| Empty string | `None` | No date found |
| Just a number "5" | `None` | No unit specified |
| Invalid format "abc" | `None` | No number found |
| "il y a" (no value) | `None` | Empty after prefix removal |

### Comparison with Other Sources

**fr.phenixscans:** Uses ISO 8601 dates via `DateTime::parse_from_rfc3339()`
**fr.rimuscans:** Similar relative date parsing (older implementation)
**fr.raijinscans:** Modern implementation with code quality improvements

### Code Quality Improvements Applied

1. ✅ Named constants instead of magic numbers
2. ✅ Explicit unit ambiguity resolution (min vs mois)
3. ✅ Edge case handling for empty units
4. ✅ Inline comments explaining parsing paths
5. ✅ Documented in this file

---

**Last Updated:** 2025-01-16
**Source Version:** 7
