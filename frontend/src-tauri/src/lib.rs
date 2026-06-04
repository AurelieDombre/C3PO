use serde::{Serialize, Deserialize};
use std::path::Path;
use walkdir::WalkDir;
use std::process::Command;
// reqwest async uniquement — blocking interdit dans le runtime Tokio de Tauri

// =========================================================
// STRUCTURES DE DONNÉES
// =========================================================

#[derive(Serialize, Clone)]
struct SearchResult {
    reply: String,
    files: Vec<FileItem>,
}

#[derive(Serialize, Deserialize, Clone)]
struct FileItem {
    name: String,
    path: String,
    file_type: String,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    score: usize,
    // Timestamp UNIX de dernière modification — tri secondaire date desc
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    modified_secs: u64,
}

#[derive(Serialize, Clone)]
struct OllamaStatus {
    installed: bool,
    running: bool,
    models: Vec<String>,
}

// =========================================================
// OLLAMA — détection de l'exécutable
// =========================================================

fn find_ollama() -> bool {
    // 1. Via le PATH (Linux / macOS / Windows si PATH à jour)
    if Command::new("ollama").arg("--version").output().is_ok() {
        return true;
    }

    // 2. Fallback Windows : emplacement par défaut de l'installeur
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let exe = format!(r"{}\Programs\Ollama\ollama.exe", local);
        if Path::new(&exe).exists() {
            return true;
        }
    }

    false
}

// =========================================================
// HELPERS — spawn sans fenêtre console (Windows)
// Rust n'autorise pas #[cfg] au milieu d'un chaînage .method(),
// on encapsule donc la logique dans des fonctions dédiées.
// =========================================================

fn spawn_ollama_serve() -> bool {
    let mut cmd = Command::new("ollama");
    cmd.arg("serve");

    // Sous Windows : CREATE_NO_WINDOW évite la console noire
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    cmd.spawn().is_ok()
}

fn spawn_ollama_pull(model: &str) -> bool {
    let mut cmd = Command::new("ollama");
    cmd.args(["pull", model]);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    cmd.spawn().is_ok()
}

// =========================================================
// COMMANDE TAURI : OLLAMA STATUS (async — ne bloque pas l'UI)
// =========================================================
#[tauri::command]
async fn ollama_status() -> OllamaStatus {

    let installed = find_ollama();

    if !installed {
        return OllamaStatus { installed: false, running: false, models: vec![] };
    }

    // Un seul appel HTTP pour running + modèles
    let resp = reqwest::Client::new()
        .get("http://127.0.0.1:11434/api/tags")
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await;

    match resp {
        Err(_) => OllamaStatus { installed: true, running: false, models: vec![] },
        Ok(r) => {
            let json: serde_json::Value = r.json().await.unwrap_or_default();
            let models = json["models"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|m| m["name"].as_str().map(String::from))
                .collect();

            OllamaStatus { installed: true, running: true, models }
        }
    }
}

// =========================================================
// COMMANDE TAURI : DÉMARRER OLLAMA
// =========================================================
#[tauri::command]
async fn start_ollama() -> bool {
    spawn_ollama_serve()
}

// =========================================================
// COMMANDE TAURI : PULL MODEL
// =========================================================
#[tauri::command]
async fn pull_model(model: String) -> bool {
    spawn_ollama_pull(&model)
}

// =========================================================
// COMMANDE TAURI : SEARCH FILES
//
// async obligatoire : Tauri 2 tourne sous Tokio.
// Appeler reqwest::blocking depuis le runtime Tokio
// (même depuis une fn non-async) bloque le runtime entier.
// On utilise spawn_blocking pour le scan fichiers (I/O
// intensive CPU/disque) et reqwest async pour Ollama.
// =========================================================
#[tauri::command]
async fn search_files(query: String, paths: Vec<String>) -> SearchResult {

    println!("Recherche : {}", query);
    println!("Paths : {:?}", paths);

    // Scan disque dans un thread dédié (blocking autorisé là)
    let query_clone = query.clone();
    let top_results = tokio::task::spawn_blocking(move || {
        scan_files(&query_clone, &paths)
    }).await.unwrap_or_default();

    println!("Top résultats : {}", top_results.len());

    // Re-rank IA via reqwest async (pas de blocking dans Tokio)
    let results = ollama_rank(&query, top_results).await;

    let reply = if results.is_empty() {
        format!("Aucun résultat trouvé pour : {}", query)
    } else {
        format!("{} résultat(s) trouvé(s)", results.len())
    };

    SearchResult { reply, files: results }
}

// Scan fichiers — appelé depuis spawn_blocking, peut utiliser
// des opérations bloquantes librement.
fn scan_files(query: &str, paths: &[String]) -> Vec<FileItem> {

    let stop_words = [
        // verbes de recherche
        "cherche", "chercher", "trouve", "trouver", "montre", "affiche",
        // articles / pronoms
        "le", "la", "les", "l", "un", "une", "des", "du", "de", "d", "mon", "ma", "mes",
        // mots génériques fichiers
        "fichier", "fichiers", "dossier", "dossiers", "document", "documents",
    ];

    // Mots temporels : signalent qu'on veut le fichier le plus récent.
    // On les retire des keywords (inutiles pour le scoring nom)
    // mais on les mémorise pour booster le tri par date.
    let recency_words = ["dernier", "dernière", "récent", "récente", "nouveau", "nouvelle", "récents", "récentes"];

    let q_lower = query.to_lowercase();

    // true si l'utilisateur veut le fichier le plus récent
    let want_latest = recency_words.iter().any(|w| q_lower.split_whitespace().any(|t| t == *w));

    let keywords: Vec<String> = q_lower
        .split_whitespace()
        .filter(|w| !stop_words.contains(w) && !recency_words.contains(w))
        // nettoie les apostrophes composées : "d'imposition" → "imposition"
        .flat_map(|w| w.split('\'').filter(|p| !p.is_empty() && !stop_words.contains(p)))
        .map(|s| s.to_string())
        .collect();

    println!("Keywords : {:?} | want_latest : {}", keywords, want_latest);

    let mut results: Vec<FileItem> = vec![];

    for base in paths {
        let base_path = Path::new(base);
        if !base_path.exists() { continue; }
        println!("Scan : {}", base);

        for entry in WalkDir::new(base_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            let full_path = path.display().to_string().to_lowercase();

            let mut score = 0;
            for kw in &keywords {
                if file_name.contains(kw) { score += 10; }
                if full_path.contains(kw)  { score += 3;  }
            }
            if score == 0 { continue; }

            let file_type = if path.is_dir() {
                "folder".to_string()
            } else {
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("file")
                    .to_string()
            };

            // Récupère la date de modification pour le tri secondaire
            let modified_secs = entry.metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            results.push(FileItem {
                name: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                path: path.display().to_string(),
                file_type,
                score,
                modified_secs,
            });
        }
    }

    // Tri :
    // - Si l'utilisateur demande "le dernier / le plus récent" → date prime
    // - Sinon → score prime, puis date en départage
    results.sort_by(|a, b| {
        if want_latest {
            b.modified_secs.cmp(&a.modified_secs)
                .then_with(|| b.score.cmp(&a.score))
        } else {
            b.score.cmp(&a.score)
                .then_with(|| b.modified_secs.cmp(&a.modified_secs))
        }
    });
    results.into_iter().take(50).collect()
}

// =========================================================
// OLLAMA IA RANKING — async (reqwest async, pas blocking)
// =========================================================
async fn ollama_rank(query: &str, files: Vec<FileItem>) -> Vec<FileItem> {
    if files.is_empty() { return files; }

    println!("Envoi à Ollama : {} fichiers", files.len());

    let input = files.iter()
        .enumerate()
        .map(|(i, f)| format!("{}: {} | {}", i, f.name, f.path))
        .collect::<Vec<_>>()
        .join("\n");

    // Prompt strict : on impose le JSON en premier, on interdit tout texte
    // autour, et on demande uniquement les indices pour éviter les
    // hallucinations sur name/path.
    let prompt = format!(
r#"You are a file search ranking engine. Respond ONLY with a JSON array, no explanation, no text before or after.

User query: {}

Numbered file list:
{}

Return a JSON array of the relevant file indices, most relevant first.
Example: [2, 0, 4]
Output ONLY the JSON array."#,
        query, input
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .unwrap_or_default();

    let response = client
        .post("http://127.0.0.1:11434/api/generate")
        .json(&serde_json::json!({
            "model": "llama3",
            "prompt": prompt,
            "stream": false,
            // temperature basse = réponses plus déterministes / moins bavardes
            "options": { "temperature": 0.1 }
        }))
        .send()
        .await;

    let Ok(response) = response else {
        println!("Ollama inaccessible pour le ranking");
        return files;
    };

    let json: serde_json::Value = match response.json().await {
        Ok(v) => v,
        Err(e) => { println!("Erreur JSON Ollama : {:?}", e); return files; }
    };

    let raw = json["response"].as_str().unwrap_or("");
    println!("Réponse Ollama :\n{}", raw);

    // Extraction robuste : on cherche le premier '[' et le dernier ']'
    // même si Ollama a mis du texte avant ou après.
    let json_str = match (raw.find('['), raw.rfind(']')) {
        (Some(start), Some(end)) if end > start => &raw[start..=end],
        _ => {
            println!("Pas de tableau JSON trouvé dans la réponse Ollama");
            return files;
        }
    };

    // Parse les indices retournés par Ollama
    match serde_json::from_str::<Vec<usize>>(json_str) {
        Ok(indices) => {
            println!("Indices Ollama : {:?}", indices);
            let ranked: Vec<FileItem> = indices.into_iter()
                .filter_map(|i| files.get(i).cloned())
                .collect();
            if ranked.is_empty() { files } else { ranked }
        }
        Err(e) => {
            println!("Erreur parsing indices JSON : {:?}", e);
            files
        }
    }
}

// =========================================================
// POINT D'ENTRÉE TAURI
// =========================================================
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(
            tauri::generate_handler![
                search_files,
                ollama_status,
                start_ollama,
                pull_model
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
