use serde::{Serialize, Deserialize};
use std::path::Path;
use walkdir::WalkDir;

// reqwest = appel HTTP vers Ollama
use reqwest::blocking::Client;

// =========================================================
// STRUCTURES DE DONNÉES
// =========================================================

// Résultat renvoyé au front React via Tauri
// → converti automatiquement en JSON
#[derive(Serialize, Clone)]
struct SearchResult {
    // message affiché dans le chat UI
    reply: String,

    // liste des fichiers trouvés
    files: Vec<FileItem>,
}

// Représente un fichier dans les résultats
#[derive(Serialize, Deserialize, Clone)]
struct FileItem {
    // nom du fichier (affichage UI)
    name: String,

    // chemin complet disque
    path: String,

    // type (pdf, folder, txt, etc.)
    file_type: String,
}

// =========================================================
// COMMANDE TAURI : SEARCH FILES
// =========================================================

// Cette fonction est exposée au front via :
// invoke("search_files", { query, paths })
#[tauri::command]
fn search_files(query: String, paths: Vec<String>) -> SearchResult {

    // log debug dans console Rust
    println!("Recherche : {}", query);
    println!("Paths : {:?}", paths);

    // =====================================================
    // 1. EXTRACTION DES MOTS CLÉS
    // =====================================================

    // "voyage italie" → ["voyage", "italie"]
    let keywords: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    // stockage des résultats brut
    let mut results: Vec<FileItem> = vec![];

    // =====================================================
    // 2. SCAN DES DOSSIERS
    // =====================================================

    for base in paths {

        let base_path = Path::new(&base);

        // ignore si dossier invalide
        if !base_path.exists() {
            continue;
        }

        // scan récursif du dossier
        for entry in WalkDir::new(base_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // récupère nom fichier en string
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            // =================================================
            // 3. MATCH DES MOTS CLÉS
            // =================================================

            let matched = keywords.iter().any(|kw| {
                file_name.contains(kw)
            });

            // si match → ajout résultat
            if matched {

                // détecte type fichier
                let file_type = if path.is_dir() {
                    "folder".to_string()
                } else {
                    path.extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("file")
                        .to_string()
                };

                results.push(FileItem {
                    name: path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string(),

                    path: path.display().to_string(),

                    file_type,
                });

                // sécurité : limite résultats
                if results.len() >= 50 {
                    break;
                }
            }
        }
    }

    // =====================================================
    // 4. OLLAMA (RE-RANK IA OPTIONNEL)
    // =====================================================

    let results = ollama_rank(&query, results);

    // =====================================================
    // 5. RÉPONSE FINALE
    // =====================================================

    let reply = if results.is_empty() {
        format!("Aucun résultat trouvé pour : {}", query)
    } else {
        format!("{} résultat(s) trouvé(s)", results.len())
    };

    SearchResult {
        reply,
        files: results,
    }
}

// =========================================================
// OLLAMA IA RANKING (API HTTP)
// =========================================================

// Cette fonction demande à Ollama de reclasser les résultats
// selon la pertinence sémantique
fn ollama_rank(query: &str, files: Vec<FileItem>) -> Vec<FileItem> {

    // si aucun résultat → pas besoin d’IA
    if files.is_empty() {
        return files;
    }

    // construit liste compacte pour prompt
    let input = files.iter()
        .take(30)
        .map(|f| format!("{} | {}", f.name, f.path))
        .collect::<Vec<_>>()
        .join("\n");

    // prompt envoyé à l’IA
    let prompt = format!(
r#"
Tu es un moteur de recherche intelligent.

Utilisateur :
{}

Voici des fichiers candidats :

{}

Retourne UNIQUEMENT du JSON valide :
[
  {{"name":"...","path":"...","file_type":"..."}}
]
"#,
        query, input
    );

    // client HTTP
    let client = Client::new();

    // appel API Ollama local
    let response = client
        .post("http://127.0.0.1:11434/api/generate")
        .json(&serde_json::json!({
            "model": "llama3",
            "prompt": prompt,
            "stream": false
        }))
        .send();

    // si erreur HTTP → fallback direct
    let Ok(response) = response else {
        return files;
    };

    // parse JSON réponse Ollama
    let json: serde_json::Value = match response.json() {
        Ok(v) => v,
        Err(_) => return files,
    };

    // texte généré par le modèle
    let raw = json["response"].as_str().unwrap_or("");

    // essaye de parser JSON IA
    match serde_json::from_str::<Vec<FileItem>>(raw) {
        Ok(v) => v,
        Err(_) => files, // fallback safe
    }
}

// =========================================================
// POINT D’ENTRÉE TAURI
// =========================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

    tauri::Builder::default()

        // permet d’exécuter des commandes système si besoin
        .plugin(tauri_plugin_shell::init())

        // ouvrir fichiers / dossiers
        .plugin(tauri_plugin_opener::init())

        // dialogues natifs (file picker etc.)
        .plugin(tauri_plugin_dialog::init())

        // expose les commandes au frontend
        .invoke_handler(
            tauri::generate_handler![
                search_files
            ]
        )

        // lance l’app Tauri
        .run(tauri::generate_context!())

        .expect("error while running tauri application");
}