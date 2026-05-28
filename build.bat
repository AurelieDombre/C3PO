@echo off

echo =========================
echo BUILD BACKEND
echo =========================

cd backend

call .venv\Scripts\activate

pyinstaller --onefile --hidden-import=api.main --hidden-import=api.schema --hidden-import=components.score --hidden-import=components.format_item --hidden-import=components.blacklist --hidden-import=uvicorn start_backend.py

echo =========================
echo COPY BACKEND
echo =========================

if not exist ..\frontend\src-tauri\bin\start_backend mkdir ..\frontend\src-tauri\bin\start_backend
copy /Y dist\start_backend.exe ..\frontend\src-tauri\bin\start_backend\start_backend.exe

cd ..

echo =========================
echo BUILD FRONTEND
echo =========================

cd frontend

npm run build

echo =========================
echo BUILD TAURI MSI
echo =========================

npm run tauri build

echo =========================
echo DONE
echo =========================

pause
