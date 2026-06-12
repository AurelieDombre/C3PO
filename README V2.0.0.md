# C3PO v2

C3PO est une application desktop locale de recherche de fichiers en langage naturel.  
L'interface est en **React/Vite**, le shell desktop en **Tauri**, le moteur de recherche en **Rust** (intégré au runtime Tauri), et **Ollama** analyse les requêtes en langage naturel avec un fallback local intégré.

> **v2 — rupture avec la v1** : le backend Python/FastAPI a été entièrement remplacé par du Rust natif compilé dans le binaire Tauri. Il n'y a plus de processus backend séparé, plus de PyInstaller, plus d'environnement virtuel Python.

---

## Sommaire

- [Prérequis](#prérequis)
- [Installation](#installation)
- [Lancer en développement](#lancer-en-développement)
- [Build de production](#build-de-production)
- [Architecture](#architecture)
- [Flux d'exécution](#flux-dexécution)
- [Commandes Tauri (API interne)](#commandes-tauri-api-interne)
- [Moteur de recherche et scoring](#moteur-de-recherche-et-scoring)
- [Limitations connues](#limitations-connues)

---

## Prérequis

| Outil | Version minimale | Rôle |
|---|---|---|
| [Node.js](https://nodejs.org) | 18+ | Toolchain frontend |
| [Rust](https://rustup.rs) | 1.77.2+ | Compilation Tauri |
| [Tauri CLI](https://tauri.app/fr/start/prerequisites/) | 2.x | Build desktop |
| [Ollama](https://ollama.com/download) | toute version récente | Analyse des requêtes (optionnel avec fallback) |

### Installer Rust

```powershell
winget install Rustlang.Rustup
rustup update stable
```

### Installer les dépendances système Tauri (Windows)

Tauri nécessite le **Microsoft C++ Build Tools** ou Visual Studio avec la charge de travail C++.  
Voir : https://tauri.app/fr/start/prerequisites/#windows

---

## Installation

```powershell
# Cloner le dépôt
git clone https://github.com/votre-user/c3po.git
cd c3po/frontend

# Installer les dépendances Node
npm install
```

Les dépendances Rust sont gérées automatiquement par Cargo au premier build.

---

## Lancer en développement

```powershell
cd frontend
npm run tauri dev
```

Cela démarre Vite en mode HMR et ouvre la fenêtre Tauri avec rechargement à chaud.  
Le moteur de recherche Rust est recompilé automatiquement si `src-tauri/src/` est modifié.

> **Note Ollama** : au premier lancement, C3PO vérifie si Ollama est installé et si un modèle est disponible. Des écrans de garde guident l'installation si besoin. Le moteur fonctionne sans Ollama grâce au parseur local intégré.

---

## Build de production

```powershell
cd frontend
npm run tauri build
```

Cela produit un installeur NSIS dans `frontend/src-tauri/target/release/bundle/nsis/`.

Le build effectue dans l'ordre :

1. `npm run build` — bundle Vite du frontend React
2. `cargo build --release` — compilation Rust du runtime Tauri + moteur de recherche
3. Packaging NSIS de l'exécutable final

> Contrairement à la v1, **il n'y a plus de `build.bat`** ni de compilation PyInstaller à lancer manuellement. Tout est orchestré par `npm run tauri build`.

---

## Architecture

```
c3po/
└── frontend/               ← projet Vite/React
    ├── src/
    │   ├── App.jsx                   ← UI principale, orchestration
    │   ├── components/
    │   │   └── OllamaGate.jsx        ← garde Ollama (install, démarrage, pull modèle)
    │   └── hooks/
    │       └── useOllama.jsx         ← état Ollama (installed/running/models)
    ├── components/
    │   └── SearchPathConfig.jsx      ← sélection des dossiers à scanner
    └── src-tauri/
        ├── src/
        │   ├── lib.rs                ← moteur Rust : recherche, scoring, Ollama
        │   └── main.rs               ← point d'entrée Tauri
        ├── Cargo.toml                ← dépendances Rust
        └── tauri.conf.json           ← config Tauri (fenêtre, bundle, ressources)
```

### Frontend React

- `App.jsx` orchestre l'UI de chat, appelle `invoke("search_files")` et gère l'ouverture native des fichiers.
- `OllamaGate.jsx` bloque l'UI tant qu'Ollama n'est pas opérationnel. Propose d'installer Ollama, de le démarrer ou de télécharger le modèle `llama3` directement depuis l'appli.
- `SearchPathConfig.jsx` permet à l'utilisateur de choisir un ou plusieurs dossiers à scanner. Les chemins sont persistés dans `localStorage["search_paths"]`.
- `useOllama.jsx` expose l'état Ollama (installed, running, models) avec rafraîchissement.

### Runtime Tauri / Rust (`lib.rs`)

C'est le cœur de la v2. Le backend Python a été entièrement réécrit en Rust et compilé dans le binaire Tauri :

- **Détection Ollama** : vérifie l'exécutable `ollama` dans le PATH et dans `%LOCALAPPDATA%\Programs\Ollama\`.
- **Démarrage Ollama** : lance `ollama serve` sans fenêtre console (flag `CREATE_NO_WINDOW` sur Windows).
- **Pull de modèle** : lance `ollama pull <modèle>` en tâche de fond.
- **Parse de requête** : envoie la requête utilisateur à `POST http://127.0.0.1:11434/api/generate` (modèle `llama3`). En cas d'échec, bascule automatiquement sur le parseur local.
- **Parseur local** : supprime les mots vides français, détecte l'intention « fichier le plus récent », split les contractions.
- **Scan de fichiers** : parcours récursif des dossiers via `walkdir`, scoring et tri des résultats.

---

## Flux d'exécution

```
Utilisateur
  └─► OllamaGate vérifie Ollama (installé ? lancé ? modèle présent ?)
        └─► UI de chat disponible
              └─► Requête utilisateur
                    └─► invoke("search_files") → Rust
                          ├─► ollama_parse() — extrait keywords + intent
                          │     └─► fallback local_parse() si Ollama KO
                          └─► scan_files() — parcours disque + scoring
                                └─► résultats triés → React
                                      └─► openPath() pour ouvrir un fichier
```

---

## Commandes Tauri (API interne)

Ces commandes sont exposées via `invoke()` côté React.

### `search_files`

Recherche des fichiers correspondant à une requête en langage naturel.

```typescript
invoke("search_files", {
  query: "mes factures EDF de l'année dernière",
  paths: ["C:\\Users\\moi\\Documents"]
})
```

Retourne :

```json
{
  "reply": "3 résultat(s) trouvé(s)",
  "files": [
    {
      "name": "facture_EDF_2024.pdf",
      "path": "C:\\Users\\moi\\Documents\\facture_EDF_2024.pdf",
      "file_type": "pdf"
    }
  ]
}
```

### `ollama_status`

Retourne l'état courant d'Ollama.

```typescript
invoke("ollama_status")
// → { installed: true, running: true, models: ["llama3:latest"] }
```

### `start_ollama`

Lance `ollama serve` en arrière-plan.

```typescript
invoke("start_ollama") // → boolean
```

### `pull_model`

Télécharge un modèle Ollama.

```typescript
invoke("pull_model", { model: "llama3" }) // → boolean
```

---

## Moteur de recherche et scoring

Le scan est récursif et non indexé (pas de base de données). Pour chaque fichier ou dossier :

**Matching** : chaque mot-clé est cherché comme mot entier (délimiteurs non-alphanumériques) dans le nom de fichier et dans le chemin complet.

**Scores** :

| Condition | Points |
|---|---|
| Mot-clé trouvé dans le nom | +20 |
| Mot-clé trouvé dans le chemin | +10 |
| Tous les mots-clés trouvés | +50 |

**Souplesse** : avec N mots-clés, `ceil(N × 0.5)` mots doivent correspondre pour qu'un fichier soit retenu.

**Tri** : par score décroissant, puis par année extraite du nom de fichier (ex. `2025` dans `avis_2025.pdf`), puis par date de modification.

**Mode "dernier fichier"** : si la requête contient un mot de récence (`dernier`, `récent`, `nouveau`…), le tri prioritise la date de modification et le résultat est limité à 5 entrées. Sans ce mode, la limite est 50 résultats.

---

## Limitations connues

- La recherche est récursive et non indexée : elle peut être lente sur de très gros dossiers.
- Le modèle Ollama ciblé est `llama3`. Pour utiliser un autre modèle, modifier `lib.rs` (`ollama_parse`).
- Les chemins sont traités en lowercase pour le matching, ce qui peut poser problème sur des systèmes de fichiers sensibles à la casse (Linux).
- Uniquement packagé pour Windows (cible NSIS dans `tauri.conf.json`).
