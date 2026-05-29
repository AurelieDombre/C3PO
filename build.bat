@echo off

echo =========================
echo BUILD BACKEND
echo =========================

cd backend

call .venv\Scripts\activate

pyinstaller --noconfirm --clean start_backend.spec

echo =========================
echo COPY BACKEND
echo =========================

robocopy dist\start_backend ..\frontend\src-tauri\bin\start_backend /MIR /NFL /NDL /NJH /NJS /NP
if %ERRORLEVEL% GEQ 8 exit /b %ERRORLEVEL%

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
