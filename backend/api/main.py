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

    query = query.lower()

    extension = None
    keyword = ""

    extensions_map = {
        "pdf": "pdf",
        "image": "png",
        "images": "png",
        "photo": "jpg",
        "photos": "jpg",
        "document": "docx",
    }

    for word, ext in extensions_map.items():

        if word in query:
            extension = ext

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

    words = query.split()

    filtered_words = [
        word for word in words
        if word not in blacklist
    ]

    keyword = " ".join(filtered_words)

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