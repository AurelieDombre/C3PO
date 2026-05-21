from fastapi import FastAPI
from pathlib import Path
from fastapi.middleware.cors import CORSMiddleware
from api.schema import Message, ChatRequest
import httpx


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


@app.get("/search")
async def search(query: str):

    ollama_response = await parse_query_with_ollama(query)

    return {
        "reply": ollama_response,
        "files": []
    }
    
    
@app.post("/chat")
async def chat(request: ChatRequest):

    # Conversion format Ollama
    ollama_messages = [
        {
            "role": m.role,
            "content": m.content
        }
        for m in request.messages
    ]

    async with httpx.AsyncClient() as client:

        response = await client.post(
            "http://localhost:11434/api/chat",
            json={
                "model": "llama3",
                "messages": ollama_messages,
                "stream": False
            },
            timeout=60.0
        )

    data = response.json()

    return {
        "reply": data["message"]["content"]
    }