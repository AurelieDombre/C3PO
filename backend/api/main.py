from fastapi import FastAPI
from pathlib import Path
from fastapi.middleware.cors import CORSMiddleware
from api.schema import Message, ChatRequest
import httpx
import json


app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/")
def root():
    return {"message": "Backend OK"}

# Recherche local sans IA
def parse_query(query: str):

    # On met tout en minuscules pour éviter les problèmes :
    query = query.lower()


    # Variable qui contiendra l'extension détectée. Exemple : "pdf"
    extension = None


    # Variable qui contiendra le mot-clé principal. Exemple : "facture"
    keyword = ""

    # Dictionnaire de traduction
    #
    # Si l'utilisateur écrit :
    # "images"
    #
    # alors on comprend :
    # "*.png"
    #
    # Tu peux ajouter plein de synonymes ici.
    extensions_map = {
        "pdf": "pdf",
        "image": "png",
        "images": "png",
        "photo": "jpg",
        "photos": "jpg",
        "document": "docx",
    }


    # On parcourt tous les mots-clés du dictionnaire. Exemple : word = "pdf";  ext = "pdf"
    # puis :
    # word = "image"
    # ext = "png"
    for word, ext in extensions_map.items():


        # Si le mot est présent dans la phrase utilisateur. Exemple :"cherche mes pdf" alors : extension = "pdf"
        if word in query:
            extension = ext

    # Liste des mots inutiles. On veut les supprimer. Exemple : "cherche mes pdf facture" devient : "facture"
    blacklist = [
        "cherche",
        "trouve",
        "mes",
        "les",
        "des",
        "de",
        "pdf",
        "images",
        "image",
        "photos",
    ]


    # On découpe la phrase en mots
    # Exemple :
    # "cherche mes pdf facture"
    # devient :
    # ["cherche", "mes", "pdf", "facture"]
    words = query.split()


    # On garde uniquement les mots utiles
    # Ici : "facture"
    # parce que les autres mots sont dans la blacklist
    filtered_words = [
        word for word in words
        if word not in blacklist
    ]


    # On reconstruit une phrase propre Exemple : ["facture"] devient : "facture"
    keyword = " ".join(filtered_words)


    # On retourne le résultat final
    # Exemple :
    # {
    #   "extension": "pdf",
    #   "keyword": "facture"
    # }
    return {
        "extension": extension,
        "keyword": keyword
    }

# Ollama
async def parse_query_with_ollama(query: str):

    prompt = f"""
Tu es un assistant qui analyse des demandes utilisateur.

Tu dois retourner UNIQUEMENT un JSON valide.

Exemple :

{{
    "intent": "search_files",
    "keywords": ["avis", "imposition"],
    "extensions": ["pdf"],
    "sort": "date_desc"
}}

Demande utilisateur :
{query}
"""

    async with httpx.AsyncClient() as client:

        response = await client.post(
            "http://localhost:11434/api/generate",
            json={
                "model": "llama3",
                "prompt": prompt,
                "stream": False
            },
            timeout=60.0
        )

    data = response.json()

    return data["response"]

def search_files(keywords: list[str], extensions: list[str], sort: str = "date_desc"):
    results = []
    base_path = Path("E:/")

    for ext in extensions:
        for file in base_path.rglob(f"*.{ext}"):
            name_lower = file.name.lower()
            # Le fichier doit contenir AU MOINS un des mots-clés
            if any(kw.lower() in name_lower for kw in keywords):
                results.append(file)
            if len(results) >= 20:
                break

    # Tri par date de modification (le plus récent en premier)
    if sort == "date_desc":
        results.sort(key=lambda f: f.stat().st_mtime, reverse=True)

    return [str(f) for f in results]

@app.get("/search")
async def search(query: str):

    ollama_response = await parse_query_with_ollama(query)

    return {
        "reply": ollama_response,
        "files": []
    }
    
    
@app.post("/chat")
async def chat(request: ChatRequest):
    # Le dernier message de l'utilisateur
    last_message = request.messages[-1].content

    # Étape 1 : Ollama analyse la phrase
    raw = await parse_query_with_ollama(last_message)

    # Étape 2 : on parse le JSON retourné par Ollama
    try:
        # Ollama ajoute parfois du texte autour du JSON, on extrait le bloc {}
        start = raw.index("{")
        end = raw.rindex("}") + 1
        parsed = json.loads(raw[start:end])
    except (ValueError, json.JSONDecodeError):
        return {"reply": "Je n'ai pas compris ta demande. Essaie : 'cherche mes pdf de factures'", "files": []}

    keywords = parsed.get("keywords", [])
    extensions = parsed.get("extensions", ["pdf"])
    sort = parsed.get("sort", "date_desc")

    # Étape 3 : recherche sur le disque
    files = search_files(keywords, extensions, sort)

    # Étape 4 : réponse formatée pour le chat
    if files:
        reply = f"J'ai trouvé {len(files)} fichier(s) correspondant à ta recherche :"
    else:
        reply = f"Aucun fichier trouvé pour les mots-clés : {', '.join(keywords)}"

    return {"reply": reply, "files": files}