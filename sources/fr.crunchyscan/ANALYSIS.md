# Analyse : CrunchyScan - Système de chargement des images

## Problème
Les images des chapitres ne se chargent pas car elles utilisent des URLs `blob:` générées dynamiquement par JavaScript.

## Analyse du HTML (data.html)

### Structure des images
```html
<div id="imageContainer" class="flex flex-col items-center mt-4 mb-4 relative">
    <img class="lazy-load h-96 imageView" style="pointer-events: none;" data-img="1">
    <img class="lazy-load h-96 imageView" style="pointer-events: none;" data-img="2">
    <!-- ... jusqu'à data-img="195" -->
</div>
```

**Observations:**
- Les images ont un attribut `data-img` (numéro de page)
- Pas d'attribut `src` dans le HTML initial
- Les URLs `blob:` sont injectées par JavaScript après chargement

### Script externe identifié
```html
<script src="/reader/reader-4598e440fa.js" defer></script>
```
Ce script est responsable du chargement des images.

### Format des URLs d'images
L'utilisateur a fourni un exemple d'URL réelle:
```
https://crunchyscan.fr/get-image?data=eyJpdiI6Ilk2Ni9kdUFBN0JrbDlxbkhBcVlmb0E9PSIsInZhbHVlIjoiaWdWemJXYStNZ3JqZ20rU0xQTDR1N0NlQWZJOHRhbFNrNGZaQmNUd3JSTmlDSi9tYkN1T0hKUVRjajhFVk9vZWVVZmpKc0dDNm9KYTYyNURTTHl1dVRkS3duNWEySEpQVjRVZjFVTitGek9Pa01TenJFREt2M3ROTjU2SGdoTnVQSlEwRktyQTNmdGFVd2NXUE0zcHJIM3BXYmszVUt6ZjRlMmFiYTdCTEJsRTZ1UUhpRFlXN2pOL3d2NVZVMDZyaHFkcTlaMDVNY29ybkVWbWFrMmRHNDlhYW1xc1RCd3lnaC9HeUlaaWFydmVBRFlyZnZ0UlZ6Rm1zTUZ5RWxtbWFKMlg4TnBreW5ZYTI4cE5LdG5zYnF6SFhabUZrd3ZPNWtjVldySzVLVUZMOVUyTm1NUlJyaHZuMHcwRzl1SWsiLCJtYWMiOiI4ZDQ0ZWI5YWRhZTM1MDdhMzVhYjdmNTBjNTYwMjM4MTJlODA1MjEzNjE4NGE3NjE5MDYyMTI5OGUyOWI4Yjg2IiwidGFnIjoiIn0%3D&expires=1764506566&signature=c3db1091a7f418c14e42245aa9c99e54bab383e098d6135f6de92ff44826275b&cid=48bbb377f72b49c2090fd12f395d4639ad95ea52692a417782b6c8c3f9d00e9c
```

### Décodage du paramètre `data`
Le paramètre `data` est du JSON encodé en base64 (format Laravel encryption):
```json
{
  "iv": "Y66/duAA7Bkl9qnHAqYfoA==",
  "value": "igVzbWa+MgrjgmxSLPL4u7CeAfI8talSk4fZBcTwrRNiCJ/mbCuOHJQTcj8EVOoeUefjJsGC6oJa625DSLyuuTdKwn5a2HJPV4Uf1UN+FzOOkMSzrEDKv3tNN56HghNuPJQ0FKrA3ftaUwcWPM3prH3pWbk3UKzf4e2aba7BLBlE6uQHiDYW7jN/wv5VU06rhqdq9Z05McornEVmak2dG49aamqsTBwygh/GyIZiarveADYrfvtRVzFmsMFyElmmaJ2X8NpkynZa28pNKtnsbqzHXZmFkwvO5kcVWrK5KUFL9U2NmMRRrhvn0w0G9uIk",
  "mac": "8d44eb9adae3507a35ab7f50c56023812e805213618476190621298e29b8b86",
  "tag": ""
}
```

### Paramètres de l'URL get-image
| Paramètre | Description |
|-----------|-------------|
| `data` | Données chiffrées (Laravel encryption) contenant probablement le chemin de l'image |
| `expires` | Timestamp d'expiration (Unix timestamp) |
| `signature` | Signature HMAC pour valider la requête |
| `cid` | ID de contexte (probablement chapter_id hashé) |

## Données trouvées dans le HTML

### Chapter ID
```html
<input type="text" class="hidden" name="chapter_id" value="480688">
```

### Informations du chapitre (JSON-LD)
```json
{
  "@type": "Chapter",
  "name": ".Volume 1",
  "url": "https://crunchyscan.fr/lecture-en-ligne/yu-gi-oh/read/dot-volume-1",
  "isPartOf": {
    "@type": "ComicSeries",
    "name": "Yu-Gi-Oh!",
    "url": "https://crunchyscan.fr/lecture-en-ligne/yu-gi-oh"
  }
}
```

## Hypothèses sur le fonctionnement

### Scénario 1: API endpoint
Il pourrait exister un endpoint API qui retourne les URLs des images:
- `GET /api/chapter/{chapter_id}/pages`
- `GET /api/manga/{slug}/chapter/{chapter_slug}/images`

### Scénario 2: Données dans le JavaScript
Le script `reader-4598e440fa.js` pourrait contenir ou charger les URLs chiffrées.

### Scénario 3: WebSocket ou Server-Sent Events
Les données pourraient être envoyées en temps réel.

## Prochaines étapes pour investigation

1. **Analyser le script reader** (`/reader/reader-4598e440fa.js`):
   - Télécharger et analyser le JavaScript
   - Chercher comment il génère les URLs

2. **Tester des endpoints API**:
   - `https://crunchyscan.fr/api/chapter/480688`
   - `https://crunchyscan.fr/api/pages/480688`

3. **Intercepter le trafic réseau** lors du chargement d'un chapitre pour voir:
   - Quelles requêtes XHR/Fetch sont faites
   - D'où viennent les URLs `get-image?data=...`

## Découvertes (via interception réseau avec Playwright)

### URLs directes des images trouvées
En interceptant le trafic réseau lors du chargement d'un chapitre, les URLs suivantes ont été identifiées :
```
https://crunchyscan.fr/upload/manga/yu-gi-oh/chapter_5713565039/LVCibePFAo7lLIYWK5M0kTGCHYI8tHccEjxwaCN6/SqCJ9ix.jpg
https://crunchyscan.fr/upload/manga/yu-gi-oh/chapter_5713565039/LVCibePFAo7lLIYWK5M0kTGCHYI8tHccEjxwaCN6/aNqacLt.jpg
```

### Pattern des URLs d'images
```
https://crunchyscan.fr/upload/manga/{manga-slug}/chapter_{internal_id}/{random_folder}/{image_name}.jpg
```

| Composant | Exemple | Description |
|-----------|---------|-------------|
| manga-slug | `yu-gi-oh` | Slug du manga |
| internal_id | `5713565039` | ID interne (différent du chapter_id HTML: 480688) |
| random_folder | `LVCibePFAo7lLIYWK5M0kTGCHYI8tHccEjxwaCN6` | Dossier avec caractères aléatoires |
| image_name | `SqCJ9ix.jpg` | Nom de fichier court aléatoire |

### Données chiffrées dans le HTML
Les données sont stockées dans un élément caché :
```html
<div id="a-ads-id" data-meta="7687d95fb967c396574fe3..."></div>
<div id="ads-inject" data-number="195"></div>
```

- **`data-meta`** : Chaîne hexadécimale contenant les données chiffrées des URLs d'images
- **`data-number`** : Nombre total de pages (195 dans cet exemple)

### Fichiers de déchiffrement
- **reader.js**: `/reader/reader-4598e440fa.js` - Script principal du reader (obfusqué)
- **crypto.wasm**: `/reader/crypto-4598e440fa.wasm` - Module WebAssembly pour le déchiffrement

### Processus de chargement
1. Le JavaScript `reader.js` charge les métadonnées du chapitre
2. Le module `crypto.wasm` déchiffre les données pour obtenir les URLs réelles
3. Les images sont chargées directement depuis `/upload/manga/...`

## Analyse complète du script reader.js

### Structure du script (obfusqué)
Le script `reader-4598e440fa.js` est fortement obfusqué avec:
- Tableau de chaînes encodées (`a0_0x7f10`)
- Fonctions de déobfuscation (`a0_0x23be`)
- Protection anti-débogage

### Clés de chiffrement identifiées
```javascript
'K0Q6YqGsxCtCLPLG'           // 16 chars - Clé XOR
'aYdjAA9bFlWzoO2ZDjvw51DUhIy9' // 28 chars - Clé XOR
'Tr3eGFZNXPT'                 // 11 chars - Clé XOR
'Da6OrC5nUgi9RqKYDa6OrC5nUgi9RqKY' // 32 chars - Clé AES?
'1234567890123412'            // 16 chars - IV AES
'L3EtGmOqE746udz0k8P74tUq'    // 24 chars - Partie clé
'3jBYzWHkXj1Gke3VcS6pLDLz'    // 24 chars - Partie clé
```

### Fonctions WASM identifiées
Le module `crypto.wasm` expose des fonctions avec des noms de jeux vidéo:
```
decrypt, tetris, mario, minecraft, roblox, pokemon, gta
```
Ces fonctions sont chaînées pour le déchiffrement.

### Algorithme de déchiffrement (reconstitué)

#### Étape 1: Conversion hex → texte
```javascript
encryptedText = dataMeta.match(/.{1,2}/g)
  .map(byte => String.fromCharCode(parseInt(byte, 16)))
  .join('');
```

#### Étape 2: Chaîne de déchiffrement XOR (via WASM ou fallback JS)
```javascript
// Chemin WASM:
result = wasm.decrypt(input, key1);
result = wasm.tetris(result, key2);
result = wasm.roblox(result, key3);
result = wasm.decrypt(result, key4);

// Chemin fallback JavaScript (si WASM échoue):
function xorDecrypt(text, key) {
  let result = '';
  for (let i = 0; i < text.length; i++) {
    result += String.fromCharCode(
      text.charCodeAt(i) ^ key.charCodeAt(i % key.length)
    );
  }
  return result;
}

function subtractDecrypt(text, key) {
  const keyLen = key.length;
  const result = Array(text.length);
  for (let i = 0; i < text.length; i++) {
    const keyChar = key.charCodeAt(i % keyLen);
    result[i] = String.fromCharCode((text.charCodeAt(i) - keyChar + 256) % 256);
  }
  return result.join('');
}

// Chaîne de déchiffrement:
step1 = subtractDecrypt(encryptedText, key1);
step2 = xorDecrypt(step1, key2);
step3 = xorDecrypt(step2, key3);
step4 = subtractDecrypt(step3, key4);
```

#### Étape 3: Déchiffrement AES-CBC final
```javascript
const decryptedBytes = new Uint8Array(step4.split('').map(c => c.charCodeAt(0)));
const iv = decryptedBytes.slice(0, 16);
const ciphertext = decryptedBytes.slice(16);

// Clé AES construite par concaténation
const aesKeyBase64 = memories(wasm, wasm.tetris()) +
                     memories(wasm, wasm.mario()) +
                     memories(wasm, wasm.pokemon()) +
                     memories(wasm, wasm.gta());
const aesKey = Uint8Array.from(atob(aesKeyBase64), c => c.charCodeAt(0));

// Déchiffrement AES-CBC
const cryptoKey = await crypto.subtle.importKey('raw', aesKey, {name: 'AES-CBC'}, false, ['decrypt']);
const decrypted = await crypto.subtle.decrypt({name: 'AES-CBC', iv: iv}, cryptoKey, ciphertext);

// Résultat: URLs séparées par ';'
const imageUrls = new TextDecoder().decode(decrypted).split(';');
```

### Problèmes pour l'implémentation Rust

1. **Clés dynamiques**: Les clés XOR sont hardcodées mais les clés AES sont générées par WASM
2. **Dépendance WASM**: Sans reverse-engineer le module WASM, impossible d'obtenir les clés AES
3. **Protection multi-couche**: 4 étapes de XOR + AES-CBC

## Conclusion finale

**Le système utilise un déchiffrement côté client en 5 étapes:**
1. Conversion hex → bytes
2. 4 opérations XOR/subtract avec clés fixes
3. Déchiffrement AES-CBC avec clé dérivée du WASM

**Problème pour Aidoku:**
- Aidoku ne peut pas exécuter JavaScript/WebAssembly
- La clé AES finale nécessite l'exécution du module WASM
- Sans cette clé, impossible de déchiffrer les URLs

**Solutions possibles:**
1. **Reverse-engineer le WASM** - Extraire les clés AES statiques (si elles le sont)
2. **Implémenter en Rust** - Possible SI les clés sont statiques
3. **Proxy externe** - Service qui déchiffre les URLs (non viable pour Aidoku)
4. **Contacter les administrateurs** - Demander un accès API direct

## Confirmation par interception réseau (Playwright)

### URLs réelles des images capturées
```
https://crunchyscan.fr/upload/manga/yu-gi-oh/chapter_5713565039/LVCibePFAo7lLIYWK5M0kTGCHYI8tHccEjxwaCN6/SqCJ9ix.jpg
https://crunchyscan.fr/upload/manga/yu-gi-oh/chapter_5713565039/LVCibePFAo7lLIYWK5M0kTGCHYI8tHccEjxwaCN6/aNqacLt.jpg
```

### Pattern confirmé
```
https://crunchyscan.fr/upload/manga/{manga-slug}/chapter_{internal_id}/{random_folder}/{image_name}.jpg
```

| Composant | Valeur | Notes |
|-----------|--------|-------|
| manga-slug | `yu-gi-oh` | Identique au slug URL |
| internal_id | `5713565039` | Différent du chapter_id HTML (480688) |
| random_folder | `LVCibePFAo7lLIYWK5M0kTGCHYI8tHccEjxwaCN6` | 40 chars, généré côté serveur |
| image_name | `SqCJ9ix.jpg` | 7 chars + extension |

### Problème fondamental
- Le `random_folder` et `image_name` sont **imprévisibles**
- Ces valeurs sont **chiffrées** dans `data-meta`
- Sans déchiffrement, impossible de construire les URLs

## Status: BLOQUÉ

La source CrunchyScan ne peut pas fonctionner dans Aidoku car:

1. **Protection par chiffrement**: Les URLs des images sont chiffrées côté serveur
2. **Déchiffrement client-side**: Nécessite JavaScript + WebAssembly
3. **Clés dynamiques**: La clé AES finale est dérivée du module WASM
4. **Aucun contournement possible**: Sans exécuter le WASM, impossible d'obtenir les URLs

### Résumé technique
```
HTML → data-meta (hex) → XOR×4 → AES-CBC → URLs séparées par ";"
                         ↑
                    Clés du WASM
                    (inaccessibles)
```

### Recommandation
Cette source devrait être déplacée vers `offline-sources/` jusqu'à ce que:
- CrunchyScan propose une API publique
- Ou désactive la protection des images
- Ou qu'Aidoku supporte l'exécution JavaScript (improbable)
