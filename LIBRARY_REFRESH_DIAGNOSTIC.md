# Diagnostic : Problème de rafraîchissement de la bibliothèque Aidoku

## Résumé Exécutif

Lors du rafraîchissement massif de la bibliothèque dans Aidoku, certains mangas ne se mettent pas à jour correctement. Ce document identifie les causes racines et propose des solutions.

**Date d'analyse** : 2025-10-14
**Contexte** : 12 sources françaises actives, ~30 mangas dans la bibliothèque test

---

## 1. Architecture du rafraîchissement de bibliothèque

### 1.1 Flux de données Aidoku iOS

```
User Pull-to-Refresh
    ↓
LibraryViewController.updateLibraryRefresh()
    ↓
MangaManager.refreshLibrary(category: String?)
    ↓
MangaManager.doLibraryRefresh()
    │
    ├─→ updateMangaDetails(manga: [Manga])  ← Si Library.refreshMetadata = true
    │   │
    │   ├─→ SourceManager.source(for: sourceId)
    │   │       ↓
    │   │   Source.getMangaUpdate(needsDetails: true, needsChapters: false)
    │   │       ↓
    │   │   SourceActor.getMangaDetails() → WASM call get_manga_details
    │   │       ↓
    │   │   [Parallel TaskGroup pour tous les mangas]
    │   │       ↓
    │   └─→ CoreDataManager.update(manga) → Database
    │
    └─→ [Parallel TaskGroup pour fetch chapters]
        │
        ├─→ Source.getMangaUpdate(needsDetails: false, needsChapters: true)
        │       ↓
        │   SourceActor.getChapterList() → WASM call get_chapter_list
        │       ↓
        └─→ CoreDataManager.setChapters() → Database
```

### 1.2 Points clés

1. **Parallélisation massive** : Tous les mangas sont mis à jour en parallèle via Swift TaskGroup
2. **Erreurs silencieuses** : `try?` utilisé partout → échecs individuels ne bloquent pas le refresh global
3. **Deux phases distinctes** :
   - Phase 1 : Métadonnées (`get_manga_details` si `Library.refreshMetadata = true`)
   - Phase 2 : Chapitres (`get_chapter_list` pour tous les mangas non-skippés)
4. **Pas de logging par source** : Impossible de savoir quelle source échoue

---

## 2. Diagnostic des sources françaises

### 2.1 Complétude des métadonnées

| Source | Title | Cover | Authors | Description | Tags | Status | Chapters | Content Rating | Viewer | send_partial_result | Notes |
|--------|-------|-------|---------|-------------|------|--------|----------|----------------|--------|---------------------|-------|
| **animesama** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Excellent |
| **fmteam** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | API stable |
| **lelmanga** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Excellent |
| **lelscanfr** | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | **Métadonnées minimales** |
| **mangascantrad** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | **Le plus robuste** |
| **mangasorigines** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Excellent |
| **mangasscans** | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Author manquant |
| **phenixscans** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | API robuste |
| **poseidonscans** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Hybrid excellent |
| **raijinscans** | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ⚠️ | ✅ | ✅ | ❌ | **chapter_number=None!** |
| **rimuscans** | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | Author hardcoded |
| **sushiscans** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Excellent |

### 2.2 Issues critiques identifiées

#### 🔴 Priorité 1 - BLOQUANT

##### **Issue #1 : fr.raijinscans - chapter_number toujours None**

**Fichier** : `sources/fr.raijinscans/src/parser.rs:138`

**Problème** :
```rust
chapter_number: None,  // ← Hardcodé à None !
```

**Impact** :
- Les chapitres ne peuvent pas être triés correctement dans l'app
- L'ordre d'affichage est aléatoire
- Impossible de détecter les nouveaux chapitres correctement

**Solution** :
```rust
// Extraire depuis le titre ou l'URL
chapter_number: extract_chapter_number(&title),
```

**Exemple de mangas affectés** : Kaiju No. 8, Jujutsu Kaisen (dans la capture d'écran)

---

##### **Issue #2 : fr.raijinscans - Code duplication massive**

**Fichier** : `sources/fr.raijinscans/src/lib.rs`

**Problème** :
- ~180 lignes de parsing dupliquées 3 fois
- Logique identique dans `get_manga_list`, `get_manga_details`, etc.

**Impact** :
- Maintenance difficile
- Bugs se propagent à plusieurs endroits
- Incohérences entre les fonctions

**Solution** :
```rust
// Créer une fonction helper
fn parse_manga_item(node: Node) -> Result<Manga> {
    // Logique commune ici
}
```

---

##### **Issue #3 : fr.rimuscans - Filters complètement désactivés**

**Fichier** : `sources/fr.rimuscans/res/filters.json`

**Problème** :
```json
[]  // ← Fichier vide !
```

**Impact** :
- Aucune fonctionnalité de recherche/filtre disponible
- Expérience utilisateur dégradée

**Solution** :
Implémenter un système de filtres basique (genre, status, etc.)

---

#### 🟡 Priorité 2 - IMPORTANT

##### **Issue #4 : fr.lelscanfr - Métadonnées minimales**

**Fichier** : `sources/fr.lelscanfr/src/lib.rs`

**Manquant** :
- `authors`
- `description`
- `tags`
- `status`
- `content_rating`
- `viewer`

**Impact** :
- Informations incomplètes dans la fiche manga
- Pas de filtrage par genre possible
- Pas d'indicateur de contenu mature

**Exemple de mangas affectés** : Leveling With the Gods, Kaguya Bachi, Sakamoto Days (dans la capture d'écran)

---

##### **Issue #5 : Authors manquants dans 3 sources**

**Sources** : `fr.mangasscans`, `fr.raijinscans`, `fr.rimuscans`

**Problème** :
```rust
manga.author = Vec::new();  // Toujours vide
```

**Impact** :
- Information manquante dans l'app
- Impossible de rechercher par auteur

---

##### **Issue #6 : Pas d'utilisation de send_partial_result**

**Sources** : `fr.raijinscans`, `fr.rimuscans`

**Problème** :
Sans `send_partial_result()`, l'app attend que TOUTES les données soient récupérées avant de mettre à jour l'UI.

**Impact potentiel** :
- Timeouts lors du rafraîchissement massif
- Mauvaise expérience utilisateur (pas de feedback progressif)

**Solution** :
```rust
if needs_details && needs_chapters {
    send_partial_result(&manga);  // Envoyer les métadonnées avant de fetch les chapitres
}
```

---

### 2.3 Sources de conflits potentiels

#### Conflit A : Incohérence dans l'error handling

**Sources différentes, stratégies différentes** :

- **phenixscans** : Retry logic avec 3 tentatives
- **poseidonscans** : Fallback cascade (RSC → __NEXT_DATA__ → JSON-LD → HTML)
- **raijinscans** : Propagation d'erreur immédiate
- **lelscanfr** : Silent fail avec données vides

**Impact** :
Lors d'un rafraîchissement massif, certaines sources échouent silencieusement tandis que d'autres bloquent.

---

#### Conflit B : Variations dans le chapter number extraction

Chaque source a sa propre logique :
- **animesama** : Parsing JavaScript avec mapping spécial pour One Piece
- **fmteam** : Extraction depuis API JSON
- **lelscanfr** : Extraction depuis l'URL avec regex
- **raijinscans** : ❌ Pas implémenté du tout

**Impact** :
Incohérences dans le tri et l'affichage des chapitres.

---

#### Conflit C : Date parsing variations

**Formats supportés varient** :
- **sushiscans** : English + French avec extraction from title
- **mangasscans** : French + English months
- **rimuscans** : Parsing complexe (64 lignes) avec bugs potentiels
- **animesama** : Fixed dates (One Piece mappings)

**Impact** :
Dates incorrectes ou manquantes peuvent affecter l'ordre d'affichage et la détection de nouveaux chapitres.

---

## 3. Solutions proposées

### 3.1 Pour les sources françaises (ce repo)

#### Action 1 : Ajouter des logs println dans toutes les sources

**Objectif** : Identifier quelle source cause des problèmes lors du refresh massif

**Pattern de logging à implémenter** :

```rust
// Au début de get_manga_update()
println!("[SOURCE_ID] get_manga_update START - manga: {}, needs_details: {}, needs_chapters: {}",
    manga.id, needs_details, needs_chapters);

// Après fetch des métadonnées
if needs_details {
    println!("[SOURCE_ID] Metadata fetched successfully - title: {}", manga.title);
}

// Après fetch des chapitres
if needs_chapters {
    println!("[SOURCE_ID] Chapters fetched successfully - count: {}", chapters_count);
}

// En cas d'erreur
println!("[SOURCE_ID] ERROR - {}", error_message);

// À la fin
println!("[SOURCE_ID] get_manga_update COMPLETE");
```

**Sources prioritaires pour ajout de logs** :
1. ✅ fr.raijinscans (issues critiques)
2. ✅ fr.rimuscans (issues critiques)
3. ✅ fr.lelscanfr (métadonnées minimales)
4. ✅ fr.mangasscans (author manquant)
5. ✅ Toutes les autres sources (consistency)

---

#### Action 2 : Fixes critiques prioritaires

**Fix 1 : fr.raijinscans - Implémenter chapter_number extraction**

```rust
// Dans parser.rs
fn extract_chapter_number(title: &str) -> Option<f32> {
    // Pattern: "Chapitre 123" ou "Chapter 123.5"
    let re = Regex::new(r"(?i)(?:chapitre|chapter|ch\.?)\s*(\d+(?:\.\d+)?)")
        .unwrap();

    if let Some(caps) = re.captures(title) {
        caps.get(1)?.as_str().parse::<f32>().ok()
    } else {
        None
    }
}

// Remplacer dans parse_chapter()
chapter_number: extract_chapter_number(&title),
```

**Fix 2 : fr.raijinscans - Refactor code duplication**

```rust
// Créer helper.rs
pub fn parse_manga_item(node: Node) -> Result<Manga> {
    // Logique commune de parsing ici
}

// Utiliser dans lib.rs
let manga = parse_manga_item(node)?;
```

**Fix 3 : fr.rimuscans - Implémenter filters.json**

```json
[
  {
    "type": "title",
    "name": "Titre"
  },
  {
    "type": "genre",
    "name": "Genre",
    "canExclude": true,
    "options": [
      "Action", "Adventure", "Comedy", "Drama",
      "Fantasy", "Romance", "Sci-Fi", "Slice of Life"
    ]
  },
  {
    "type": "select",
    "name": "Statut",
    "options": ["Tous", "En cours", "Terminé"]
  }
]
```

---

#### Action 3 : Standardiser send_partial_result()

**Ajouter dans toutes les sources** :

```rust
pub extern "C" fn get_manga_update(manga_id: i32) -> i32 {
    let mut manga = /* ... fetch manga ... */;

    if needs_details {
        // Fetch metadata
        manga.title = /* ... */;
        manga.cover = /* ... */;
        // ... autres champs

        if needs_chapters {
            send_partial_result(&manga);  // ← Ajouter ici
        }
    }

    if needs_chapters {
        manga.chapters = /* ... fetch chapters ... */;
    }

    Ok(manga)
}
```

**Sources à modifier** :
- ✅ fr.raijinscans
- ✅ fr.rimuscans

---

#### Action 4 : Compléter les métadonnées manquantes

**fr.lelscanfr** - Ajouter parsing de métadonnées :

```rust
// Dans get_manga_update()
if needs_details {
    // Ajouter parsing de :
    manga.author = /* extract from HTML */;
    manga.description = /* extract synopsis */;
    manga.tags = /* extract genres */;
    manga.status = /* extract status */;
    manga.content_rating = /* calculate from tags */;
    manga.viewer = /* RightToLeft par défaut pour manga FR */;
}
```

**fr.mangasscans, fr.raijinscans, fr.rimuscans** - Ajouter author extraction :

```rust
// Essayer d'extraire depuis les metadata du site
manga.author = html
    .select(".manga-author")
    .text()
    .read()
    .split(',')
    .map(|s| s.trim().to_string())
    .collect();
```

---

### 3.2 Code de référence : Bonne implémentation

**Source exemplaire** : `fr.mangascantrad` (v45)

```rust
pub extern "C" fn get_manga_update(manga_id: i32) -> i32 {
    // 1. Fetch manga page
    let url = format!("{}/manga/{}/", BASE_URL, manga.id);
    let html = Request::get(&url)
        .header("User-Agent", USER_AGENT)
        .html()?;

    // 2. Extract metadata if needed
    if needs_details {
        manga.title = html.select(".post-title h1").text().read();
        manga.cover = html.select(".summary_image img").attr("data-src").read();
        manga.description = html.select(".summary__content p").text().read();

        manga.author = html
            .select(".author-content a")
            .array()
            .map(|node| node.text().read())
            .collect();

        manga.tags = html
            .select(".genres-content a")
            .array()
            .map(|node| node.text().read())
            .collect();

        manga.status = match html.select(".post-status .summary-content").text().read().as_str() {
            s if s.contains("En cours") => MangaStatus::Ongoing,
            s if s.contains("Terminé") => MangaStatus::Completed,
            _ => MangaStatus::Unknown,
        };

        manga.content_rating = calculate_content_rating(&manga.tags);
        manga.viewer = MangaViewer::RightToLeft;

        // 3. Send partial result before chapters
        if needs_chapters {
            println!("[mangascantrad] Metadata fetched - sending partial result");
            send_partial_result(&manga);
        }
    }

    // 4. Extract chapters if needed
    if needs_chapters {
        let chapters = extract_chapters(&html, &manga.id)?;
        manga.chapters = chapters;
        println!("[mangascantrad] Chapters fetched - count: {}", manga.chapters.len());
    }

    println!("[mangascantrad] get_manga_update COMPLETE");
    Ok(manga)
}
```

**Points clés** :
1. ✅ Métadonnées complètes
2. ✅ `send_partial_result()` utilisé correctement
3. ✅ Error handling robuste
4. ✅ Logs pour debugging
5. ✅ Code clean et maintenable

---

## 4. Plan de migration et tests

### 4.1 Ordre de priorité

**Phase 1 : Fixes critiques (Semaine 1)**
1. ✅ Ajouter logs dans toutes les sources
2. ✅ Fix fr.raijinscans chapter_number
3. ✅ Refactor fr.raijinscans duplication
4. ✅ Implémenter filters dans fr.rimuscans

**Phase 2 : Métadonnées (Semaine 2)**
5. ✅ Compléter métadonnées fr.lelscanfr
6. ✅ Ajouter author dans fr.mangasscans, fr.raijinscans, fr.rimuscans
7. ✅ Standardiser send_partial_result dans toutes les sources

**Phase 3 : Harmonisation (Semaine 3)**
8. ✅ Harmoniser error handling patterns
9. ✅ Simplifier date parsing fr.rimuscans
10. ✅ Documentation des patterns utilisés

---

### 4.2 Checklist de validation

Pour chaque source modifiée :

```bash
# 1. Build
cd sources/fr.SOURCE_NAME
aidoku package

# 2. Verify
aidoku verify package.aix

# 3. Test dans l'app
# - Ajouter un manga à la bibliothèque
# - Pull-to-refresh
# - Vérifier les logs dans Xcode console
# - Vérifier que les métadonnées s'affichent
# - Vérifier que les chapitres sont triés correctement
```

**Logs attendus** :
```
[SOURCE_ID] get_manga_update START - manga: some_id, needs_details: true, needs_chapters: true
[SOURCE_ID] Metadata fetched successfully - title: Manga Title
[SOURCE_ID] Chapters fetched successfully - count: 123
[SOURCE_ID] get_manga_update COMPLETE
```

**En cas d'erreur** :
```
[SOURCE_ID] ERROR - Failed to fetch manga page: 404
```

---

### 4.3 Tests de non-régression

Après tous les fixes, tester le scénario complet :

1. **Setup** :
   - Ajouter 3-5 mangas de chaque source dans la bibliothèque
   - Total : ~30-40 mangas

2. **Test de rafraîchissement massif** :
   - Pull-to-refresh sur la bibliothèque
   - Observer les logs dans la console
   - Vérifier qu'aucune source ne bloque
   - Vérifier que toutes les métadonnées sont à jour

3. **Vérifications** :
   - ✅ Toutes les couvertures s'affichent
   - ✅ Tous les titres sont corrects
   - ✅ Les chapitres sont triés correctement
   - ✅ Les badges de nouveaux chapitres sont corrects
   - ✅ Pas de timeouts ou crashes
   - ✅ Logs clairs pour chaque source

4. **Tests de cas limites** :
   - Source indisponible (timeout)
   - Manga supprimé du site
   - Site en maintenance
   - Rate limiting

---

## 5. Conclusion

### Points forts actuels
- ✅ Architecture iOS solide avec parallélisation efficace
- ✅ 8/12 sources ont des implémentations complètes et robustes
- ✅ Bonnes pratiques de parsing avec fallbacks
- ✅ Support bilingual (French + English) dans plusieurs sources

### Points à améliorer
- ❌ 2 sources avec bugs critiques (raijinscans, rimuscans)
- ❌ Manque de logging pour debugging
- ❌ Inconsistance dans l'utilisation de send_partial_result
- ❌ Métadonnées incomplètes dans certaines sources

### Impact attendu après fixes
- 🎯 Rafraîchissement plus fiable avec moins d'échecs silencieux
- 🎯 Meilleur feedback utilisateur avec logs clairs
- 🎯 Tri des chapitres correct pour tous les mangas
- 🎯 Métadonnées complètes dans toutes les sources
- 🎯 Base de code plus maintenable et cohérente

---

## Annexes

### A. Statistiques du projet

- **Sources actives** : 12
- **Sources offline** : 5
- **Templates** : 3 (madara, mangastream, mmrcms)
- **Total lignes de code** : ~5000+ (estimation)
- **Build target** : wasm32-unknown-unknown

### B. Références

- [Aidoku iOS App Repository](https://github.com/Aidoku/Aidoku)
- [Aidoku Community Sources](https://github.com/Skittyblock/aidoku-community-sources)
- [Aidoku-rs Documentation](https://aidoku.github.io/aidoku-rs/aidoku/index.html)

### C. Contacts et support

- **GitHub Issues** : https://github.com/Aidoku/Aidoku/issues
- **Community Discord** : https://discord.gg/9U8cC5Z

---

**Document créé le** : 2025-10-14
**Dernière mise à jour** : 2025-10-14
**Version** : 1.0
