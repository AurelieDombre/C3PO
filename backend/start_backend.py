import uvicorn

uvicorn.run(
    "api.main:app",
    host="localhost",
    port=8000,
    reload=False
)