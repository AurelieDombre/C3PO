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


@app.get("/search")
def search(query: str):
    
    parsed = parse_query(query)
    extension = parsed["extension"]
    keyword = parsed["keyword"]
    
    results = []
    
    # S'il n'y a pas d'extension => tableau vide
    if extension is None:
        return {"files": []}

    for file in Path("E:/").rglob(f"*.{extension}"):

        if keyword.lower() in file.name.lower():
            results.append(str(file))

        if len(results) >= 20:
            break

    return {"files": results}


@app.post("/chat")
async def chat(request: ChatRequest):
    # On convertit en format Ollama
    ollama_messages = [{"role": m.role, "content": m.content} for m in request.messages]
    
    async with httpx.AsyncClient() as client:
        response = await client.post(
            "http://localhost:11434/api/chat",
            json={
                "model": "llama3",  
                "messages": ollama_messages,
                "stream": False,
            },
            timeout=60.0,
        )
    
    data = response.json()
    return {"reply": data["message"]["content"]}