from fastapi import FastAPI
from pathlib import Path
from fastapi.middleware.cors import CORSMiddleware

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


@app.get("/search")
def search(extension: str, keyword: str = ""):
    results = []

    for file in Path("E:/").rglob(f"*.{extension}"):

        if keyword.lower() in file.name.lower():
            results.append(str(file))

        if len(results) >= 20:
            break

    return {"files": results}

