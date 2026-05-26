from fastapi import FastAPI
from pathlib import Path
from fastapi.middleware.cors import CORSMiddleware
from api.schema import ChatRequest
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

    # Liste des mots inutiles. On veut les supprimer. Exemple : "cherche mes pdf facture" devient : "facture"
    blacklist = {
        "cherche",
        "trouve",
        "moi",
        "mon",
        "mes",
        "le",
        "la",
        "les",
        "de",
        "des",
        "du",
        "un",
        "une",
        "et",
        "ou",
        "dans",
        "sur",
        "avec",
        "pour",
    }
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
    #   "keyword": "facture"
    # }
    return {
        "keywords": keywords if keywords else ["*"],
        "sort": "date_desc",
        "limit": 20
    }

# =========================================================
# #. OLLAMA PARSER 
# =========================================================
async def parse_query_with_ollama(query: str):

    # Prompt envoyé au modèle LLM (Ollama)
    # Objectif : forcer une réponse STRICTEMENT en JSON
    prompt = f"""
    Return ONLY valid JSON:

    Exemple :

    {{
        "intent": "search_files",
        "keywords": ["avis", "imposition"],
        "sort": "date_desc",
    }}

    Règles :
    - Si l'utilisateur demande "dernier", "plus récent", "latest", alors :
        "sort": "date_desc",
        "last": true,
        "limit": 1

    Demande utilisateur :
    {query}
    """

    try:
        # Création d’un client HTTP asynchrone avec timeout de sécurité parce qu'il faut que fastapi soit pret
        async with httpx.AsyncClient(timeout=5.0) as client:

            # Envoi de la requête POST vers l'API Ollama locale
            response = await client.post(
                "http://127.0.0.1:11434/api/generate",
                json={
                    "model": "llama3",   # modèle utilisé
                    "prompt": prompt,    # prompt construit ci-dessus
                    "stream": False      # réponse complète (pas en streaming)
                },
                timeout=60.0  # timeout global de la requête (sécurité)
            )

        # Vérifie si le serveur a répondu correctement (status HTTP 200)
        response.raise_for_status()

        # Convertit la réponse HTTP en JSON Python
        data = response.json()

        # Récupère le texte généré par le modèle
        # (souvent du texte contenant du JSON + parfois du bruit)
        raw = data.get("response", "")

        # =========================
        # EXTRACTION DU JSON
        # =========================

        # Cherche le premier "{"
        start = raw.find("{")

        # Cherche le dernier "}"
        end = raw.rfind("}") + 1

        # Si aucun JSON détecté → on abandonne proprement
        if start == -1 or end == 0:
            return None

        # Extraction de la partie JSON dans la chaîne
        json_string = raw[start:end]

        # Conversion du JSON texte en dictionnaire Python
        return json.loads(json_string)

    except Exception as e:
        # Capture toutes les erreurs possibles :
        # - Ollama offline
        # - timeout réseau
        # - JSON invalide
        # - erreur HTTP
        print(f"⚠️ Ollama error ({type(e).__name__}): {e}")

        # Retour sécurisé : fallback vers parser local
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
def search_files(
    keywords: list[str],
    sort: str = "date_desc",
    limit: int = 20
):

    # Liste des résultats trouvés
    results = []

    # Dossier racine à scanner
    base_path = Path("E:/")

    # Sécurité :
    # si aucun mot-clé → wildcard
    keywords = keywords or ["*"]

    # Scan récursif de TOUS les fichiers
    for file in base_path.rglob("*"):

        # Ignore les dossiers
        if not file.is_file():
            continue

        # Nom du fichier en minuscule
        # pour comparaison insensible à la casse
        name_lower = file.name.lower()

        # Score de pertinence du fichier
        score = 0

        # WILDCARD
        # Si "*" présent :
        # on accepte tous les fichiers
        if "*" in keywords:

            score = 1

        else:

            # CALCUL DU SCORE

            for kw in keywords:

                kw = kw.lower()

                # Ignore mots trop courts
                if len(kw) <= 2:
                    continue

                # Mot-clé présent dans le nom
                if kw in name_lower:

                    # Score de base
                    score += 10

                    # Bonus si le fichier commence par le mot
                    if name_lower.startswith(kw):
                        score += 20

        # Ignore les fichiers sans pertinence
        if score == 0:
            continue

        # Ajout du fichier + score
        results.append((score, file))

    # TRI DES RÉSULTATS
    # Tri par pertinence puis date
    if sort == "date_desc":

        results.sort(
            key=lambda x: (
                -x[0],                  # score DESC
                -x[1].stat().st_mtime  # date DESC
            )
        )

    # Limite après tri
    results = results[:limit]

    # FORMAT FINAL

    # Pour chaque chemin de fichier :
    # création d’un lien ouvrable
    return [
        {
            "name": file.name,
            "path": str(file),
            
            # URL compatible navigateur / frontend
            #"url": f"file:///{str(file).replace('\\', '/')}",

            # Debug / pertinence
            "score": score
        }

        for score, file in results
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
        parsed.get("sort"),
        parsed.get("limit"),
    )

    return build_search_response(files, parsed)
    

