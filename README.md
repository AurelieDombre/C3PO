# C3PO

C3PO est une application desktop locale de recherche de fichiers.
L'interface est en React/Tauri, le moteur de recherche en FastAPI/Python, et Ollama sert a parser les requetes en langage naturel.

Ce document decrit l'etat actuel du projet tel qu'il existe dans ce depot.

## Vue d'ensemble

- Frontend desktop React/Vite.
- Shell desktop Tauri pour les fonctions natives.
- Backend FastAPI qui scanne le systeme de fichiers.
- Integration Ollama pour l'analyse de la requete.
- Fallback local cote backend si Ollama ne repond pas.

## Architecture

```text
Utilisateur
  -> Frontend React/Tauri
  -> Backend FastAPI local
  -> Recherche systeme de fichiers
  -> Retour de resultats + ouverture native
```

### Frontend

Le frontend vit dans `frontend/`.

- `frontend/src/App.jsx` orchestre l'UI de chat, l'appel au backend et l'ouverture native des fichiers.
- `frontend/src/components/BackendGate.jsx` attend que `GET /health` reponde.
- `frontend/src/components/OllamaGate.jsx` bloque l'UI si Ollama n'est pas detecte.
- `frontend/components/SearchPathConfig.jsx` permet de choisir un ou plusieurs dossiers a scanner.
- `frontend/src/hooks/useBackendStatus.jsx` et `frontend/src/hooks/useOllama.jsx` gerent les etats de disponibilite.

### Tauri

Le shell desktop est dans `frontend/src-tauri/`.

- `frontend/src-tauri/src/lib.rs` verifie Ollama, demarre le backend local et le surveille.
- `frontend/src-tauri/src/main.rs` lance l'application Tauri.
- `frontend/src-tauri/tauri.conf.json` declare les ressources embarquees et la fenetre desktop.

### Backend

Le backend vit dans `backend/`.

- `backend/api/main.py` expose les endpoints HTTP.
- `backend/api/schema.py` definit le schema de la requete chat.
- `backend/components/score.py` attribue un score aux fichiers et dossiers trouves.
- `backend/components/format_item.py` normalise la reponse fichier.
- `backend/components/blacklist.py` contient les mots ignores pendant le parsing local.
- `backend/start_backend.py` lance Uvicorn.
- `backend/start_backend.spec` sert au packaging PyInstaller.

## Flux d'execution

1. Tauri demarre.
2. Le runtime Rust verifie Ollama sur `127.0.0.1:11434`.
3. Le runtime Rust lance le backend compile depuis `frontend/src-tauri/bin/start_backend/`.
4. Le frontend attend que `GET http://127.0.0.1:8000/health` reponde.
5. L'utilisateur ajoute un ou plusieurs dossiers a scanner.
6. La requete utilisateur est envoyee a `POST /chat`.
7. Le backend tente d'abord l'analyse via Ollama, puis bascule sur le parseur local si besoin.
8. Le backend scanne les fichiers, trie les resultats et renvoie une liste ouvrable nativement.

## API actuelle

### `GET /`

Retourne:

```json
{ "message": "Backend OK" }
```

### `GET /health`

Retourne:

```json
{ "status": "ok" }
```

### `POST /chat`

Corps attendu:

```json
{
  "messages": [
    { "role": "user", "content": "cherche mes factures" }
  ],
  "paths": ["D:/Documents"]
}
```

Reponse typique:

```json
{
  "reply": "J'ai trouve 3 resultat(s) correspondant a ta recherche.",
  "files": [
    {
      "name": "facture.pdf",
      "path": "D:\\Documents\\facture.pdf",
      "type": "file",
      "score": 120
    }
  ]
}
```

## Recherche et scoring

Le backend ne construit pas encore d'index. Il parcourt recursivement les dossiers demandes.

- Les mots courants sont retires avec la blacklist.
- Les correspondances exactes sur le nom valent plus que les correspondances partielles.
- Les dossiers gagnent un bonus par rapport aux fichiers a score egal.
- Le tri final se fait par score decroissant puis date de modification decroissante.
- Si le meilleur resultat est un dossier dont le nom correspond exactement au mot recherche, le backend peut ne renvoyer que ce dossier.

Les regles de scoring sont dans `backend/components/score.py`.

## Etat des dependances

### Frontend

Le frontend utilise:

- React 19
- Vite
- `@tauri-apps/api`
- `@tauri-apps/plugin-dialog`
- `@tauri-apps/plugin-opener`
- `@tauri-apps/plugin-shell`

### Backend

Le backend utilise:

- FastAPI
- Uvicorn
- httpx
- pydantic

### IA locale

Ollama est utilise comme parseur local de requetes.
Le code cible actuellement le modele `llama3`.

## Build et execution

### Backend seul

```powershell
cd backend
py -m venv .venv
.venv\Scripts\activate
pip install -r requirements.txt
python start_backend.py
```

### Frontend desktop

Le runtime desktop Tauri attend un binaire backend deja compile dans `frontend/src-tauri/bin/start_backend/`.
La chaine de build actuelle est decrite dans `build.bat`.

```powershell
build.bat
```

Le script:

1. compile le backend avec PyInstaller,
2. copie le dossier `dist/start_backend` dans `frontend/src-tauri/bin/start_backend`,
3. lance le build frontend,
4. lance le build Tauri.

## Limites connues

- L'UI bloque si Ollama n'est pas detecte, meme si le backend contient deja un parseur local de secours.
- `frontend/components/SearchPathConfig.jsx` lit `localStorage["paths"]` au demarrage, mais ecrit dans `localStorage["search_paths"]`.
- La recherche est recursive et non indexee, donc elle peut etre lente sur de gros dossiers.
- Le comportement de tri utilise des chemins Windows.
- Le backend doit etre present en binaire pour le runtime desktop Tauri.

## Fichiers a lire en premier

- `frontend/src/App.jsx`
- `frontend/src-tauri/src/lib.rs`
- `backend/api/main.py`
- `backend/components/score.py`
- `frontend/components/SearchPathConfig.jsx`

