from fastapi import FastAPI
from pathlib import Path


app = FastAPI()

@app.get("/")
def root():
    return {"message": "Backend OK"}


@app.get("/search")
def search(extension: str):

    results = []

    for file in Path("D:/").rglob(f"*.{extension}"):
        results.append(str(file))

        if len(results) >= 20:
            break

    return {"files": results}

