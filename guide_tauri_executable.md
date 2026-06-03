# 📦 Guide complet — Transformer C3PO en exécutable avec Tauri

> **Projet de référence** : C3PO Assistant Local — React + FastAPI + Ollama  
> **Objectif** : comprendre chaque étape pour produire un `.exe` installable sous Windows  
> **Niveau** : débutant/intermédiaire — chaque concept est expliqué avant le code

---

## Vue d'ensemble de l'architecture

```
┌─────────────────────────────────────────┐
│              TAURI (.exe)               │
│                                         │
│  ┌──────────────────────────────────┐   │
│  │     Frontend React (Vite)        │   │
│  │  App.jsx · hooks · components    │   │
│  └────────────────┬─────────────────┘   │
│                   │ fetch HTTP          │
│  ┌────────────────▼─────────────────┐   │
│  │   Backend Python (FastAPI)       │   │
│  │   start_backend.exe (PyInstaller)│   │
│  └────────────────┬─────────────────┘   │
│                   │                     │
│         ┌─────────┴──────────┐          │
│         ▼                    ▼          │
│    Filesystem            Ollama         │
│    (recherche)         (LLM local)      │
└─────────────────────────────────────────┘
```

**Ce que fait Tauri** : il sert de "boîte" qui embarque le frontend React et lance le backend Python en arrière-plan. L'utilisateur final installe un seul `.exe` et tout fonctionne.

---

## Table des matières

1. [Prérequis — Ce qu'il faut installer](#1-prérequis)
2. [Structure du projet](#2-structure-du-projet)
3. [Partie 1 — Le Backend Python avec PyInstaller](#3-partie-1--le-backend-python-avec-pyinstaller)
4. [Partie 2 — Le Frontend React + Vite](#4-partie-2--le-frontend-react--vite)
5. [Partie 3 — Tauri : la coquille desktop](#5-partie-3--tauri-la-coquille-desktop)
6. [Partie 4 — Le cœur de Tauri : lib.rs expliqué ligne par ligne](#6-partie-4--le-cœur-de-tauri--librs-expliqué)
7. [Partie 5 — Communiquer entre Rust et React (invoke)](#7-partie-5--communiquer-entre-rust-et-react-invoke)
8. [Partie 6 — Les permissions Tauri (capabilities)](#8-partie-6--les-permissions-tauri-capabilities)
9. [Partie 7 — Le build final avec build.bat](#9-partie-7--le-build-final)
10. [Partie 8 — Les erreurs fréquentes et comment les résoudre](#10-partie-8--erreurs-fréquentes)
11. [Récapitulatif — Le flux complet en un coup d'œil](#11-récapitulatif)

---

## 1. Prérequis

Avant de commencer, tu dois avoir installé ces outils sur ta machine **Windows**.

### 1.1 Node.js (≥ 18)

Tauri CLI est un paquet npm. Node est donc indispensable.

```bash
# Vérifie ta version
node --version   # doit afficher v18.x ou plus
npm --version
```

Télécharge depuis : https://nodejs.org

---

### 1.2 Rust (via rustup)

Tauri est écrit en Rust. Tu as besoin du compilateur Rust pour produire l'exécutable.

```bash
# Installation de rustup (gestionnaire de Rust)
# Va sur https://rustup.rs et exécute le script proposé

# Vérifie après installation
rustc --version     # ex : rustc 1.77.2
cargo --version     # ex : cargo 1.77.2
```

> **Pourquoi Rust ?** Tauri utilise Rust pour la partie "native" de l'application : gestion des fenêtres, accès au système de fichiers, lancement de processus. C'est ce qui rend Tauri ultra-léger comparé à Electron.

---

### 1.3 Visual Studio Build Tools (Windows uniquement)

Rust sur Windows a besoin des outils de compilation C++.

- Télécharge **Build Tools for Visual Studio** sur https://visualstudio.microsoft.com/visual-cpp-build-tools/
- Coche : **"Développement Desktop en C++"**

---

### 1.4 WebView2 (Windows)

Tauri utilise le moteur WebView2 de Microsoft pour afficher le frontend. Sur Windows 11, il est préinstallé. Sur Windows 10, il peut être absent.

```bash
# Vérifie s'il est présent
# Cherche "Microsoft Edge WebView2 Runtime" dans les programmes installés
# Sinon télécharge depuis : https://developer.microsoft.com/microsoft-edge/webview2/
```

---

### 1.5 Python (≥ 3.9) + pip

Pour le backend FastAPI et PyInstaller.

```bash
python --version  # ex : Python 3.11.x
pip --version
```

---

### 1.6 Installation du Tauri CLI

Une fois Node installé, dans le dossier `frontend/` :

```bash
npm install
# Le package.json inclut déjà @tauri-apps/cli en devDependency
# Donc npm install suffit

# Vérifie
npx tauri --version  # ex : tauri-cli 2.11.2
```

---

## 2. Structure du projet

Voici l'arborescence de C3PO telle qu'elle existe, et le rôle de chaque élément :

```
c3po assistant local/
│
├── backend/                        ← Code Python (FastAPI)
│   ├── api/
│   │   ├── main.py                 ← API FastAPI (routes /chat, /health)
│   │   └── schema.py               ← Modèles Pydantic (ChatRequest, Message)
│   ├── components/
│   │   ├── blacklist.py            ← Mots à ignorer dans les requêtes
│   │   ├── format_item.py          ← Formatage des résultats fichiers
│   │   └── score.py                ← Calcul de pertinence des fichiers
│   ├── requirements.txt            ← Dépendances Python
│   ├── start_backend.py            ← Point d'entrée PyInstaller
│   └── start_backend.spec          ← Config PyInstaller
│
├── frontend/                       ← Code React + config Tauri
│   ├── src/                        ← Code source React
│   │   ├── App.jsx                 ← Composant principal
│   │   ├── main.jsx                ← Point d'entrée React
│   │   ├── components/
│   │   │   ├── BackendGate.jsx     ← Attend que le backend soit prêt
│   │   │   └── OllamaGate.jsx      ← Vérifie si Ollama est installé
│   │   └── hooks/
│   │       ├── useBackendStatus.jsx ← Poll /health toutes les 2s
│   │       └── useOllama.jsx        ← Appelle invoke("is_ollama_available")
│   ├── components/
│   │   └── SearchPathConfig.jsx    ← Sélection des dossiers à scanner
│   ├── src-tauri/                  ← Code Rust (Tauri)
│   │   ├── src/
│   │   │   ├── main.rs             ← Point d'entrée Rust
│   │   │   └── lib.rs              ← Logique principale Tauri
│   │   ├── capabilities/
│   │   │   └── default.json        ← Permissions accordées au frontend
│   │   ├── icons/                  ← Icônes de l'application
│   │   ├── Cargo.toml              ← Dépendances Rust
│   │   └── tauri.conf.json         ← Configuration principale Tauri
│   ├── package.json                ← Dépendances npm
│   └── vite.config.js              ← Config Vite (bundler frontend)
│
└── build.bat                       ← Script de build tout-en-un
```

---

## 3. Partie 1 — Le Backend Python avec PyInstaller

### 3.1 Pourquoi PyInstaller ?

Tauri est une application native : elle ne peut pas "lancer Python" directement sur la machine de l'utilisateur final (qui n'a peut-être pas Python installé). La solution : **compiler le backend Python en un .exe autonome** avec PyInstaller.

```
start_backend.py  →  PyInstaller  →  start_backend.exe
                                     + dossier _internal/
                                     (toutes les dépendances Python)
```

Ce dossier sera ensuite **copié dans Tauri** pour être embarqué dans l'installateur final.

---

### 3.2 Le fichier `start_backend.py`

C'est le **point d'entrée** de ton backend. PyInstaller a besoin d'un fichier racine pour savoir quoi compiler.

```python
# backend/start_backend.py
from api.main import app          # Importe l'app FastAPI

if __name__ == "__main__":
    import uvicorn

    uvicorn.run(
        app,
        host="127.0.0.1",         # Écoute uniquement en local (sécurité)
        port=8000                  # Port fixe, connu par le frontend
    )
```

> **Pourquoi `host="127.0.0.1"` ?** Parce que le backend ne doit pas être accessible depuis l'extérieur, seulement depuis l'application locale. `localhost` peut parfois pointer vers IPv6 (`::1`) et causer des erreurs de connexion refusée — c'est pourquoi `App.jsx` force aussi `127.0.0.1` explicitement.

---

### 3.3 Le fichier `start_backend.spec`

Le fichier `.spec` est la **configuration avancée de PyInstaller**. Plutôt que de passer 20 arguments en ligne de commande, tout est décrit ici.

```python
# backend/start_backend.spec
a = Analysis(
    ['start_backend.py'],          # Fichier d'entrée
    pathex=[],
    binaries=[],
    datas=[],
    hiddenimports=[                # ← CRUCIAL : modules que PyInstaller ne détecte pas seul
        'api.main',
        'api.schema',
        'components.score',
        'components.format_item',
        'components.blacklist',
        'uvicorn',
        'uvicorn.logging',
        'uvicorn.loops',
        'uvicorn.loops.auto',
        'uvicorn.protocols',
        'uvicorn.protocols.http',
        'uvicorn.protocols.http.auto',
        'uvicorn.protocols.websockets',
        'uvicorn.lifespan',
        'uvicorn.lifespan.on',
    ],
    ...
)
```

**Pourquoi `hiddenimports` ?**

PyInstaller analyse le code statiquement pour trouver les imports. Mais uvicorn charge beaucoup de modules **dynamiquement** (via des strings comme `importlib.import_module("uvicorn.loops.auto")`). PyInstaller ne les voit pas. Si tu oublies ces imports, ton `.exe` plantera au démarrage avec une erreur `ModuleNotFoundError`.

```python
pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,        # ← Mode COLLECT (pas onefile)
    name='start_backend',
    debug=False,
    console=False,                # ← Pas de fenêtre console noire
    ...
)

coll = COLLECT(
    exe,
    a.binaries,
    a.zipfiles,
    a.datas,
    strip=False,
    upx=True,
    name='start_backend',         # ← Produit dist/start_backend/
)
```

**Mode COLLECT vs onefile :**

| Mode | Ce que ça produit | Avantage |
|------|-------------------|----------|
| `--onefile` | Un seul `start_backend.exe` | Simple |
| COLLECT (défaut ici) | Dossier `start_backend/` avec l'exe + `_internal/` | Démarrage plus rapide, pas de décompression au lancement |

C3PO utilise le mode COLLECT car uvicorn et FastAPI avec leurs dépendances font ~50-100 Mo — le mode `onefile` décompresserait tout ça à chaque lancement (lent).

---

### 3.4 Préparer l'environnement virtuel Python

```bash
cd backend

# Crée un environnement virtuel isolé
python -m venv .venv

# L'activer (Windows)
.venv\Scripts\activate

# Installer les dépendances
pip install -r requirements.txt
# requirements.txt contient : fastapi, uvicorn, httpx, pydantic

# Installer PyInstaller
pip install pyinstaller
```

---

### 3.5 Lancer PyInstaller

```bash
# Depuis backend/, avec .venv activé
pyinstaller --noconfirm --clean start_backend.spec
```

Résultat après compilation :

```
backend/
└── dist/
    └── start_backend/
        ├── start_backend.exe     ← L'exécutable
        └── _internal/            ← Python + toutes les dépendances
            ├── python3XX.dll
            ├── uvicorn/
            └── ... (des centaines de fichiers)
```

> **`--noconfirm`** : ne demande pas de confirmation pour écraser les fichiers existants.  
> **`--clean`** : supprime le cache de build précédent pour repartir propre.

---

### 3.6 Tester le backend compilé

Avant de continuer, vérifie que ton `.exe` fonctionne :

```bash
# Lance le backend compilé
dist\start_backend\start_backend.exe

# Dans un autre terminal, teste l'API
curl http://127.0.0.1:8000/health
# Réponse attendue : {"status":"ok"}
```

---

## 4. Partie 2 — Le Frontend React + Vite

### 4.1 Structure React de C3PO

Le frontend a une architecture en "portes" (Gates) : avant d'afficher l'interface principale, deux vérifications sont faites :

```
App.jsx
  └── BackendGate          ← "Le backend Python est-il prêt ?"
        └── OllamaGate     ← "Ollama est-il installé ?"
              └── <App>    ← Interface principale du chatbot
```

Cette approche est très propre : chaque composant a une seule responsabilité.

---

### 4.2 `useBackendStatus.jsx` — Surveiller le backend

```jsx
// src/hooks/useBackendStatus.jsx
import { useEffect, useState } from "react";

export default function useBackendStatus() {
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let interval;

    async function check() {
      try {
        // Appelle /health sur le backend Python
        const res = await fetch("http://127.0.0.1:8000/health");
        if (res.ok) setReady(true);   // Passe à true dès que le backend répond
      } catch {
        setReady(false);              // Backend pas encore prêt (en cours de démarrage)
      }
    }

    check();                          // Vérification immédiate
    interval = setInterval(check, 2000); // Puis toutes les 2 secondes

    return () => clearInterval(interval); // Nettoyage au démontage du composant
  }, []);

  return ready;
}
```

**Pourquoi ce polling ?** Le backend Python (`start_backend.exe`) est lancé par Rust dès l'ouverture de l'app, mais il prend ~1-2 secondes à démarrer. Sans ce mécanisme, le frontend essaierait de faire des requêtes avant que le serveur soit prêt.

---

### 4.3 `BackendGate.jsx`

```jsx
// src/components/BackendGate.jsx
import useBackendStatus from "../hooks/useBackendStatus.jsx";

export default function BackendGate({ children }) {
  const ready = useBackendStatus();

  if (!ready) {
    // Affiche un écran de chargement pendant le démarrage du backend
    return (
      <div style={{ padding: 20 }}>
        <h2>Initialisation backend...</h2>
      </div>
    );
  }

  // Quand le backend est prêt, affiche les composants enfants
  return children;
}
```

---

### 4.4 `useOllama.jsx` — Communiquer avec Rust via `invoke`

C'est ici qu'intervient la magie Tauri : le **bridge Rust ↔ JavaScript**.

```jsx
// src/hooks/useOllama.jsx
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";  // ← Import Tauri

export function useOllama() {
  const [status, setStatus] = useState({
    loading: true,
    available: false
  });

  useEffect(() => {
    invoke("is_ollama_available")         // ← Appelle une fonction Rust
      .then((res) => {
        setStatus({ loading: false, available: res });
      })
      .catch(() => {
        setStatus({ loading: false, available: false });
      });
  }, []);

  return status;
}
```

`invoke("is_ollama_available")` appelle directement une fonction définie en Rust dans `lib.rs`. C'est le mécanisme central de Tauri pour que le frontend accède aux capacités natives. Nous détaillerons cela en [Partie 5](#7-partie-5--communiquer-entre-rust-et-react-invoke).

---

### 4.5 `vite.config.js`

```js
// frontend/vite.config.js
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  // Note : pas de configuration server.port ici
  // Tauri attend par défaut sur le port 5173
})
```

En mode développement (`npm run dev`), Vite lance un serveur sur `http://localhost:5173`. Tauri pointe sur cette URL (voir `tauri.conf.json` → `devUrl`).

En mode production (`npm run build`), Vite génère les fichiers statiques dans `dist/`. Tauri les embarque directement (voir `frontendDist`).

---

## 5. Partie 3 — Tauri, la coquille desktop

### 5.1 `tauri.conf.json` — Le fichier de configuration principal

C'est **le fichier le plus important de Tauri**. Il définit tout : le nom de l'app, la fenêtre, ce qui est embarqué, comment builder.

```jsonc
// frontend/src-tauri/tauri.conf.json
{
  "$schema": "../node_modules/@tauri-apps/cli/config.schema.json",
  
  "productName": "C3PO",          // Nom affiché dans la barre de titre et l'installateur
  "version": "0.1.6",             // Version de l'application
  "identifier": "com.c3po.assistant",  // Identifiant unique (comme un package Android)

  "build": {
    "frontendDist": "../dist",           // Où Vite met les fichiers buildés
    "devUrl": "http://localhost:5173",   // URL du serveur Vite en dev
    "beforeBuildCommand": "npm run build" // Commande lancée avant tauri build
  },

  "app": {
    "windows": [
      {
        "title": "C3PO",
        "width": 400,            // Largeur initiale de la fenêtre
        "height": 600,           // Hauteur initiale
        "resizable": true        // L'utilisateur peut redimensionner
      }
    ],
    "security": {
      "csp": null                // Content Security Policy désactivée (développement)
                                 // En production, il faudrait la définir !
    }
  },

  "bundle": {
    "active": true,
    "targets": ["nsis"],         // Type d'installateur : NSIS = .exe Windows
    "resources": ["bin/start_backend/"], // ← Dossier embarqué dans l'installateur
    "icon": [
      "icons/icon.ico"           // Icône de l'application
    ]
  }
}
```

**Le champ `resources` est crucial** : il dit à Tauri d'inclure le dossier `bin/start_backend/` (le backend Python compilé) dans l'installateur. Sans ça, le backend ne sera pas présent sur le PC de l'utilisateur.

**Cibles de build disponibles :**

| Cible | Format | Système |
|-------|--------|---------|
| `nsis` | `.exe` installateur | Windows |
| `msi` | `.msi` installateur | Windows |
| `deb` | `.deb` paquet | Linux |
| `appimage` | `.AppImage` | Linux |
| `dmg` | `.dmg` | macOS |

---

### 5.2 `Cargo.toml` — Les dépendances Rust

```toml
# frontend/src-tauri/Cargo.toml
[package]
name = "app"
version = "0.1.0"
edition = "2021"
rust-version = "1.77.2"

[lib]
name = "app_lib"
crate-type = ["staticlib", "cdylib", "rlib"]
# staticlib : bibliothèque statique (pour iOS)
# cdylib    : bibliothèque dynamique (pour Android)
# rlib      : bibliothèque Rust normale (pour desktop)

[build-dependencies]
tauri-build = { version = "2.6.2", features = [] }

[dependencies]
serde_json = "1.0"              # Sérialisation JSON
serde = { version = "1.0", features = ["derive"] }
log = "0.4"                     # Macros de log
tauri = { version = "2.11.2", features = [] }

# Plugins Tauri utilisés dans lib.rs
tauri-plugin-log = "2"
tauri-plugin-shell = "2.3.5"    # Permet d'ouvrir des URLs (OllamaGate)
tauri-plugin-opener = "2"       # Ouvre des fichiers avec leur app native
tauri-plugin-dialog = "2.7.1"   # Boîte de dialogue de sélection de fichier
```

---

### 5.3 `main.rs` — Point d'entrée minimaliste

```rust
// frontend/src-tauri/src/main.rs

// Empêche l'ouverture d'une fenêtre console noire sous Windows en release
// NE PAS SUPPRIMER cette ligne !
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Délègue tout à lib.rs
    app_lib::run();
}
```

Ce fichier est intentionnellement minimal. Toute la logique est dans `lib.rs`. Cette séparation est la convention Tauri pour permettre des builds multi-plateformes (notamment mobile).

---

## 6. Partie 4 — Le cœur de Tauri : `lib.rs` expliqué

C'est le fichier le plus complexe. Voici une explication ligne par ligne.

### 6.1 Structure générale

```rust
pub fn run() {
    // Tout se passe à l'intérieur de run()
    
    tauri::Builder::default()
        .plugin(...)           // Enregistrement des plugins
        .invoke_handler(...)   // Déclaration des commandes appelables depuis React
        .setup(|app| {         // Code exécuté au démarrage de l'app
            // 1. Initialisation du log
            // 2. Vérification d'Ollama
            // 3. Lancement du backend Python
            // 4. Thread de monitoring du backend
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running app");
}
```

---

### 6.2 Le système de log

```rust
let log_path = "C:/Users/Public/c3po_debug.log";
let mut log = OpenOptions::new()
    .create(true)    // Crée le fichier s'il n'existe pas
    .append(true)    // Ajoute à la fin (ne supprime pas l'existant)
    .open(log_path)
    .ok();           // .ok() convertit Result en Option (ignore les erreurs d'ouverture)

// Macro personnalisée pour écrire dans le fichier
macro_rules! log {
    ($($arg:tt)*) => {
        if let Some(ref mut f) = log {
            use std::io::Write;
            let _ = writeln!(f, $($arg)*);
        }
    };
}

log!("=== C3PO démarrage ===");
```

**Pourquoi un log dans `C:/Users/Public/` ?** C'est un dossier accessible en écriture par tous les utilisateurs Windows, sans droits admin. Très pratique pour déboguer une app packagée.

---

### 6.3 Vérification d'Ollama

```rust
// Vérifie si le port 11434 répond (port standard d'Ollama)
fn is_ollama_running() -> bool {
    std::net::TcpStream::connect("127.0.0.1:11434").is_ok()
}

// Attend jusqu'à max_tries * 500ms qu'Ollama démarre
fn wait_for_ollama(max_tries: u32) -> bool {
    for _ in 0..max_tries {
        if std::net::TcpStream::connect("127.0.0.1:11434").is_ok() {
            return true;
        }
        thread::sleep(Duration::from_millis(500));
    }
    false
}

// Si Ollama n'est pas déjà lancé, on attend 5 * 500ms = 2.5 secondes max
let ollama_available = is_ollama_running();
let ollama_ready = if ollama_available { true } else { wait_for_ollama(5) };

// Stocke le résultat dans l'état partagé de l'app
// app.manage() rend cet état accessible à toutes les commandes invoke
app.manage(OllamaState { available: ollama_ready });
```

---

### 6.4 Résolution du chemin du backend

C'est l'une des parties les plus délicates. En mode packagé, les fichiers se trouvent dans un dossier temporaire créé par Tauri (différent de l'emplacement d'installation).

```rust
let backend_path = [
    "bin/start_backend/start_backend.exe",   // Chemin relatif dans les resources
    "start_backend.exe",                      // Fallback si dans la racine
]
.into_iter()
.find_map(|candidate| {
    match app
        .path()
        .resolve(candidate, tauri::path::BaseDirectory::Resource)
        // BaseDirectory::Resource = dossier des resources embarquées par Tauri
    {
        Ok(path) => {
            if path.exists() {
                Some(path)   // On a trouvé le fichier !
            } else {
                None         // Ce chemin n'existe pas, on essaie le suivant
            }
        }
        Err(_) => None,
    }
})
.expect("backend introuvable");  // Panic si aucun chemin ne fonctionne
```

**`BaseDirectory::Resource`** : quand Tauri installe une app, les fichiers `resources` sont copiés dans un dossier spécifique (souvent `AppData\Local\<appname>` ou à côté de l'exe). `app.path().resolve()` calcule le chemin absolu correct.

---

### 6.5 Lancement du backend

```rust
let child = Command::new(&backend_path)
    .current_dir(&backend_dir)  // Important : le backend cherche ses fichiers relativement à lui-même
    .stdout(Stdio::null())       // On ne capture pas la sortie standard
    .stderr(Stdio::null())       // Ni les erreurs (elles iraient dans le log Tauri)
    .spawn();                    // Lance le processus en arrière-plan (non-bloquant)

match child {
    Ok(c) => {
        log!("Backend spawné (PID {})", c.id());
        *process_ref.lock().unwrap() = Some(c);    // Sauvegarde le handle du processus
        let ok = wait_for_backend(20);             // Attend 20 * 250ms = 5 secondes max
        log!("Port 8000 joignable : {}", ok);
    }
    Err(e) => {
        log!("ERREUR spawn : {:?}", e);
        // L'app continue même si le backend ne démarre pas
        // Le BackendGate côté React restera bloqué sur "Initialisation..."
    }
}
```

---

### 6.6 Le thread de monitoring (auto-restart)

```rust
thread::spawn(move || {
    let mut retries = 0;
    loop {
        thread::sleep(Duration::from_millis(2000));  // Vérifie toutes les 2 secondes

        if std::net::TcpStream::connect("127.0.0.1:8000").is_ok() {
            retries = 0;   // Le backend est vivant, reset le compteur
            continue;
        }

        // Le backend ne répond plus !
        if retries >= 3 { break; }  // Abandon après 3 tentatives
        retries += 1;

        // Kill l'ancien processus (s'il existe encore)
        if let Ok(mut guard) = process_ref2.lock() {
            if let Some(child) = guard.as_mut() {
                let _ = child.kill();
            }
            *guard = None;
        }

        // Relance le backend
        Command::new(&backend_path2)
            .current_dir(&backend_dir2)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            ...
    }
});
```

Ce thread "watchdog" tourne pendant toute la vie de l'application. Si le backend Python crashe (mémoire, erreur), il est automatiquement relancé. C'est une technique de **résilience** essentielle pour les apps de production.

---

### 6.7 L'état Ollama et la commande invoke

```rust
// Structure qui stocke l'état d'Ollama
#[derive(Clone)]
struct OllamaState {
    available: bool,
}

// La commande appelable depuis React
// L'attribut #[tauri::command] est OBLIGATOIRE
#[tauri::command]
fn is_ollama_available(state: tauri::State<OllamaState>) -> bool {
    state.available   // Retourne simplement le booléen stocké au démarrage
}
```

Et dans `run()`, il faut **déclarer** cette commande :

```rust
.invoke_handler(tauri::generate_handler![is_ollama_available])
//                                        ↑ nom de la fonction Rust
```

Sans ce `invoke_handler`, l'appel `invoke("is_ollama_available")` côté React échouera silencieusement.

---

## 7. Partie 5 — Communiquer entre Rust et React (invoke)

### 7.1 Le principe

Tauri crée un **bridge IPC (Inter-Process Communication)** entre le code Rust natif et le code JavaScript du frontend.

```
JavaScript (React)              Rust (lib.rs)
─────────────────               ─────────────
invoke("ma_commande", args)  ↔  #[tauri::command]
                                 fn ma_commande(args) { ... }
```

### 7.2 Déclarer une commande Rust

```rust
// lib.rs

// Attribut obligatoire
#[tauri::command]
fn saluer(nom: String) -> String {
    format!("Bonjour, {} !", nom)
}

// Déclaration dans le builder
.invoke_handler(tauri::generate_handler![
    saluer,
    is_ollama_available,
    // ajoute d'autres commandes ici
])
```

### 7.3 Appeler depuis React

```jsx
import { invoke } from "@tauri-apps/api/core";

// Appel simple
const message = await invoke("saluer", { nom: "Alice" });
// → "Bonjour, Alice !"

// Avec gestion d'erreur
try {
    const result = await invoke("ma_commande", { param: "valeur" });
    console.log(result);
} catch (error) {
    console.error("Erreur invoke:", error);
}
```

### 7.4 Types supportés

| Rust | JavaScript |
|------|-----------|
| `String` | `string` |
| `bool` | `boolean` |
| `i32`, `f64`, etc. | `number` |
| `Vec<T>` | `Array` |
| `HashMap<String, T>` | `Object` |
| Struct avec `#[derive(Serialize)]` | `Object` |

---

## 8. Partie 6 — Les permissions Tauri (capabilities)

### 8.1 Pourquoi les capabilities ?

Tauri 2.x adopte un modèle de **sécurité par défaut** : le frontend n'a accès à rien par défaut. Chaque capacité doit être explicitement accordée.

C'est comme les permissions sur Android : une app doit déclarer qu'elle veut accéder à la caméra, la localisation, etc.

### 8.2 `default.json` de C3PO

```jsonc
// src-tauri/capabilities/default.json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default permissions",
  "windows": ["main"],         // S'applique à la fenêtre "main"
  "permissions": [
    "core:default",            // Permissions de base Tauri
    "shell:default",           // Plugin shell (de base)
    "shell:allow-open",        // Permet d'ouvrir une URL dans le navigateur
                               // (utilisé par OllamaGate pour ouvrir ollama.com)
    {
      "identifier": "opener:allow-open-path",
      "allow": [{ "path": "**" }]  // Ouvre n'importe quel fichier avec son app native
                                    // (bouton 📄 dans le chat)
    },
    "dialog:default"           // Boîte de dialogue pour choisir un dossier
                               // (SearchPathConfig)
  ]
}
```

### 8.3 Correspondance permissions ↔ code

| Permission | Utilisée dans | Permet |
|------------|---------------|--------|
| `shell:allow-open` | `OllamaGate.jsx` → `open("https://ollama.com/download")` | Ouvrir une URL |
| `opener:allow-open-path` | `App.jsx` → `openPath(file.path)` | Ouvrir un fichier |
| `dialog:default` | `SearchPathConfig.jsx` → `open({ directory: true })` | Choisir un dossier |

---

## 9. Partie 7 — Le build final

### 9.1 `build.bat` — Script tout-en-un

```bat
@echo off

echo === BUILD BACKEND ===

cd backend
call .venv\Scripts\activate              :: Active l'environnement Python
pyinstaller --noconfirm --clean start_backend.spec   :: Compile le backend

echo === COPY BACKEND ===

:: Copie dist/start_backend/ → frontend/src-tauri/bin/start_backend/
robocopy dist\start_backend ..\frontend\src-tauri\bin\start_backend /MIR /NFL /NDL /NJH /NJS /NP
:: /MIR = synchronisation parfaite (miroir)
:: /NFL /NDL /NJH /NJS /NP = moins de verbosité

if %ERRORLEVEL% GEQ 8 exit /b %ERRORLEVEL%   :: Arrêt si robocopy échoue

cd ..

echo === BUILD FRONTEND ===

cd frontend
npm run build           :: Vite génère dist/ (HTML + JS + CSS optimisés)

echo === BUILD TAURI ===

npm run tauri build     :: Tauri construit l'exécutable final
                        :: = npx tauri build

echo === DONE ===
pause
```

### 9.2 Ce que produit `npm run tauri build`

```
frontend/
└── src-tauri/
    └── target/
        └── release/
            ├── app.exe                          ← Exécutable direct (sans installateur)
            └── bundle/
                └── nsis/
                    └── C3PO_0.1.6_x64-setup.exe  ← ← ← Installateur NSIS
```

C'est le fichier `C3PO_x.x.x_x64-setup.exe` que tu distribues !

### 9.3 Ce que fait l'installateur NSIS

Quand l'utilisateur lance le setup :
1. Il copie l'exe Tauri dans `Program Files` (ou `AppData`)
2. Il décompresse les `resources` (= `bin/start_backend/`) à côté
3. Il crée un raccourci sur le bureau et dans le menu démarrer
4. Il enregistre l'application pour pouvoir la désinstaller

---

### 9.4 Lancer en mode développement (sans builder)

Pour développer rapidement sans relancer le build complet :

**Terminal 1 — Backend Python :**
```bash
cd backend
.venv\Scripts\activate
python start_backend.py
# Le backend tourne sur http://127.0.0.1:8000
```

**Terminal 2 — Tauri dev :**
```bash
cd frontend
npm run tauri dev
# Ouvre la fenêtre Tauri avec hot-reload React
# Rust se recompile si tu modifies lib.rs
```

En mode `dev`, Tauri pointe sur `http://localhost:5173` (le serveur Vite). Le hot-reload fonctionne : modifie un `.jsx` et la fenêtre se met à jour instantanément.

---

## 10. Partie 8 — Erreurs fréquentes

### ❌ `ModuleNotFoundError: No module named 'uvicorn.lifespan'`

**Cause** : module dynamique non déclaré dans `hiddenimports`.

**Solution** : ajoute le module manquant dans `start_backend.spec` :
```python
hiddenimports=['uvicorn.lifespan.on', 'uvicorn.lifespan.off'],
```

---

### ❌ `backend introuvable` (panic au démarrage)

**Cause** : le dossier `bin/start_backend/` n'existe pas dans `src-tauri/`.

**Solution** : vérifie que `robocopy` (ou copie manuelle) a bien copié le dossier :
```
frontend/src-tauri/bin/start_backend/start_backend.exe  ← doit exister
frontend/src-tauri/bin/start_backend/_internal/         ← doit exister
```

Et dans `tauri.conf.json` :
```json
"resources": ["bin/start_backend/"]
```

---

### ❌ `invoke` retourne une erreur `Command not found`

**Cause** : la commande n'est pas déclarée dans `invoke_handler`.

**Solution** : dans `lib.rs` :
```rust
.invoke_handler(tauri::generate_handler![
    is_ollama_available,     // ← assure-toi que la fonction est listée ici
])
```

---

### ❌ `Connection refused` sur `http://127.0.0.1:8000`

**Causes possibles :**
1. Le backend n'a pas démarré — consulte `C:/Users/Public/c3po_debug.log`
2. Problème de chemin `BaseDirectory::Resource` — le `.exe` backend n'est pas trouvé
3. Le fichier `.exe` backend est bloqué par l'antivirus

**Diagnostic** :
```
# Regarde le log
notepad C:\Users\Public\c3po_debug.log
```

---

### ❌ `tauri build` échoue avec une erreur Rust

**Cause** : Rust doit recompiler toutes les dépendances, ça peut prendre 5-15 minutes la première fois.

**Solution** : patience pour le premier build. Les suivants sont beaucoup plus rapides (cache Cargo).

Si l'erreur est de compilation :
```bash
cd frontend/src-tauri
cargo check     # Vérifie le code Rust sans builder
```

---

### ❌ La fenêtre s'ouvre mais reste blanche

**Cause** : le frontend React n'est pas buildé (`dist/` absent ou vide).

**Solution** :
```bash
cd frontend
npm run build    # Génère dist/
```

Puis vérifie que `tauri.conf.json` pointe bien vers `"frontendDist": "../dist"`.

---

### ❌ Permission refusée (`opener:allow-open-path`)

**Cause** : la permission n'est pas dans `capabilities/default.json`.

**Solution** : ajoute-la comme dans l'exemple ci-dessus (Partie 6).

---

## 11. Récapitulatif

### Le flux de build complet

```
1. cd backend
   └── .venv\Scripts\activate
   └── pyinstaller --noconfirm --clean start_backend.spec
       → dist/start_backend/ (exe + _internal/)

2. robocopy dist/start_backend ../frontend/src-tauri/bin/start_backend /MIR
   → Le backend compilé est maintenant dans Tauri

3. cd frontend && npm run build
   → dist/ (HTML + JS + CSS optimisés par Vite)

4. npm run tauri build
   → src-tauri/target/release/bundle/nsis/C3PO_x.x.x_x64-setup.exe ✅
```

### Ce qui se passe au lancement de l'app (runtime)

```
1. L'utilisateur double-clique sur C3PO.exe
2. Tauri (Rust) démarre
3. lib.rs::run() s'exécute :
   a. Vérifie si Ollama tourne (port 11434)
   b. Lance start_backend.exe (FastAPI sur port 8000)
   c. Attend que le port 8000 répond
   d. Lance le thread watchdog
4. La fenêtre s'ouvre avec le frontend React
5. React → BackendGate → poll /health → "prêt !"
6. React → OllamaGate → invoke("is_ollama_available") → true/false
7. Si Ollama absent → propose le lien de téléchargement
8. Si Ollama présent → interface du chatbot affichée ✅
```

### Checklist avant de builder

- [ ] `python --version` → ≥ 3.9
- [ ] `rustc --version` → ≥ 1.77
- [ ] `node --version` → ≥ 18
- [ ] `.venv` créé et activé dans `backend/`
- [ ] `pip install -r requirements.txt && pip install pyinstaller` fait
- [ ] `npm install` fait dans `frontend/`
- [ ] `backend/dist/start_backend/start_backend.exe` existe après PyInstaller
- [ ] `frontend/src-tauri/bin/start_backend/start_backend.exe` existe après robocopy
- [ ] `tauri.conf.json` → `"resources": ["bin/start_backend/"]`
- [ ] `lib.rs` → `.invoke_handler(tauri::generate_handler![is_ollama_available])`
- [ ] `capabilities/default.json` → toutes les permissions nécessaires présentes

---

*Guide basé sur le projet C3PO Assistant Local V1.6 — React 19 + FastAPI + Ollama + Tauri 2.11*