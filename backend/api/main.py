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

# =========================================================
# #. PARSER LOCAL (sans IA)
# =========================================================
def parse_query(query: str):

    # On met tout en minuscules pour éviter les problèmes :
    query = query.lower()

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

    # Variable qui contiendra l'extension détectée. Exemple : "pdf"
    extension = None

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
    keywords = [word for word in words if word not in blacklist]

    # On retourne le résultat final
    # Exemple :
    # {
    #   "extension": "pdf",
    #   "keyword": "facture"
    # }
    return {
        "keywords": keywords if keywords else ["*"],
        "extensions": [extension] if extension else ["pdf"],
        "sort": "date_desc",
        "limit": 20
    }

# =========================================================
# #. OLLAMA PARSER 
# =========================================================
async def parse_query_with_ollama(query: str):

    prompt = f"""
Return ONLY valid JSON:

Exemple :

{{
    "intent": "search_files",
    "keywords": ["avis", "imposition"],
    "extensions": ["pdf"],
    "sort": "date_desc",
}}

Règles :
- Si l'utilisateur demande "dernier", "plus récent", "latest", alors :
    "sort": "date_desc",
    "last": true,
    "limit": 1

- Si aucun type de fichier n'est précisé :
    utilise ["pdf"]

Demande utilisateur :
{query}
"""
    try:
        async with httpx.AsyncClient(timeout=5.0) as client:

            response = await client.post(
                "http://127.0.0.1:11434/api/generate",
                json={
                    "model": "llama3",
                    "prompt": prompt,
                    "stream": False
                },
                timeout=60.0
            )
        response.raise_for_status()
        data = response.json()
        
        raw = data.get("response", "")

        # extraction JSON SAFE
        start = raw.find("{")
        end = raw.rfind("}") + 1

        if start == -1 or end == 0:
            return None

        return json.loads(raw[start:end])
    except Exception as e:
        print(f"⚠️ Ollama error ({type(e).__name__}): {e}")
        return None

# 
# =========================================================
# #. ANALYSE UNIFIÉE (Analyse la demande et retourne la query pour ollama ou syst local) 
# =========================================================
async def analyze_query(query: str):
    parsed = await parse_query_with_ollama(query)
    # CAS 1 : Ollama OK
    if parsed:
        try:
            return {
                "keywords": parsed.get("keywords") or ["*"],
                "extensions": parsed.get("extensions") or ["pdf"],
                "sort": parsed.get("sort", "date_desc"),
                "limit": parsed.get("limit", 20),
            }
        except Exception as e:
            print("⚠️ Ollama JSON invalid:", e)
            
    # CAS 2 : fallback local garanti    
    parsed = parse_query(query)
    print("🧠 Using local parser fallback")
    return parse_query(query)

# =========================================================
# #. RECHERCHE FICHIERS
# =========================================================
def search_files(keywords: list[str], extensions: list[str], sort: str = "date_desc", limit: int = 20):
    results = []
    base_path = Path("E:/")

    keywords = keywords or ["*"]
    extensions = extensions or ["pdf"]
    
    for ext in extensions:
        for file in base_path.rglob(f"*.{ext}"):
            name_lower = file.name.lower()
            
            # wildcard = tout accepter
            if "*" in keywords:
                results.append(file)
                continue
            
            # Le fichier doit contenir AU MOINS un des mots-clés
            if any(kw.lower() in name_lower for kw in keywords):
                results.append(file)
                
    # Tri par date de modification (le plus récent en premier)
    if sort == "date_desc":
        results.sort(key=lambda f: f.stat().st_mtime, reverse=True)
        
    #Limite après trie
    results = results[:limit]

    # Pour chaque chemin de fichier créer un lien 
    return [
        {
            "name": f.name,
            "path": str(f),
            "url": f"file:///{str(f).replace('\\', '/')}"
        }
        for f in results
    ]


        
# =========================================================
# #. FORMAT DE RÉPONSE
# =========================================================
def build_search_response(files, parsed):

    keywords = parsed.get("keywords", [])

    if files:
        reply = (
            f"J'ai trouvé {len(files)} fichier(s) "
            f"correspondant à ta recherche."
        )
    else:
        reply = (
            "Aucun fichier trouvé pour : "
            + ", ".join(keywords)
        )
    return {
        "reply": reply,
        "files": files
    }


# =========================================================
# #. ENDPOINT PRINCIPAL CHAT
# =========================================================    
@app.post("/chat")
async def chat(request: ChatRequest):
    
# Le dernier message de l'utilisateur
    last_message = request.messages[-1].content
    
# analyse de la demande
    parsed = await analyze_query(last_message)
    
#recherche sur le disque
    files = search_files(
        parsed.get("keywords"),
        parsed.get("extensions"),
        parsed.get("sort"),
        parsed.get("limit"),
    )

    return build_search_response(files, parsed)
    

