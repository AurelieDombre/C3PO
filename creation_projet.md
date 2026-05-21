# Process d'initialisation d'un chatBot

## Créer le projet :

1. Créer le repositorie
   
Soit créer le repo dans gitHub et cloner en local :

`git clone git@github.com:Identifiant/repository.git`

 soit créer le dossier en local et lier a github en remote :

```shell
git init
git add README.md
git commit -m "first commit"
git branch -M main ou master
git remote add origin https://github.com/Identifiant/repository.git
git push -u origin main ou master
```

2. Créer l'architecture du projet

assistant-local/
├── backend/ 
└── README.md

1. Créer l'environnement de pyton pour le backend

Dans le dossier backend, il faut générer l'environnement .venv : `python -m venv .venv`
Active le :`` .\.venv\Scripts\Activate``

Puis lancer le serveur :

```shell
cd backend
uvicorn main:app --reload
```

4. Créer le frontend avec Vite

Pour un frontend moderne avec react, il faut créer le dossier frontend avec vite :
`npm create vite@latest frontend`
 puis choisir : React et javascript dans les technos proposées.

Lancer le serveur avec : `npm run dev`
On doit avoir une page web sur http://localhost:5173/ avec un get started de vite :emoji:

## Installation et configuration

### Dans le backend:

1. **Installer** FastAPI et Uvicorn : ``pip install fastapi uvicorn``

Uvicorn et FastAPI, le rôle d’Uvicorn est de faire tourner ton application FastAPI.

FastAPI ne sait pas “écouter Internet” tout seul.
C’est juste le framework qui définit tes routes (/users, /login, etc.).

Uvicorn est le serveur ASGI qui :
   - démarre l’application,
   - écoute les requêtes HTTP,
   - les transmet à FastAPI,
   - renvoie les réponses au client.

2. **Initialiser le back**

Créer un dossier api dans le dossier backend puis un fichier main.py. Ce dernier va contenir l’ensemble de nos endpoints pour communiquer avec l'api

```python
from fastapi import FastAPI

app = FastAPI()

@app.get("/")
def root():
    return {"message": "Backend OK"}
```
Si on clique sur le lien générer par le serveur : http://127.0.0.1:8000
on doit avoir un retour json :
```json
{
"message": "Backend OK"
}
```

