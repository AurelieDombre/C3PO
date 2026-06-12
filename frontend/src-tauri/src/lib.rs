use serde::{Serialize, Deserialize};
use std::path::Path;
use walkdir::WalkDir;
use std::process::Command;

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
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    modified_secs: u64,
    // Année extraite du nom du fichier (ex: "2025" dans "avis 2025.pdf")
    // Prioritaire sur modified_secs pour want_latest
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    year_in_name: u32,
}

#[derive(Serialize, Clone)]
struct OllamaStatus {
    installed: bool,
    running: bool,
    models: Vec<String>,
}

// Ce qu'Ollama extrait de la requête utilisateur
#[derive(Deserialize, Debug)]
struct ParsedQuery {
    // mots-clés à chercher dans les noms de fichiers
    keywords: Vec<String>,
    // true si l'utilisateur veut uniquement le fichier le plus récent
    want_latest: bool,
}

// =========================================================
// OLLAMA — détection de l'exécutable
// =========================================================

fn find_ollama() -> bool {
    let mut cmd = Command::new("ollama");
    cmd.arg("--version");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    if cmd.output().is_ok() {
        return true;
    }
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
// =========================================================

fn spawn_ollama_serve() -> bool {
    let mut cmd = Command::new("ollama");
    cmd.arg("serve");
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
// COMMANDE TAURI : OLLAMA STATUS
// =========================================================
#[tauri::command]
async fn ollama_status() -> OllamaStatus {
    let installed = find_ollama();
    if !installed {
        return OllamaStatus { installed: false, running: false, models: vec![] };
    }
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
// OLLAMA : PARSE LA REQUÊTE UTILISATEUR
//
// Ollama sert uniquement ici — il extrait les keywords
// et l'intention. Il ne voit jamais la liste de fichiers.
// Fallback local si Ollama ne répond pas.
// =========================================================
async fn ollama_parse(query: &str) -> ParsedQuery {

    let prompt = format!(
r#"You are a search query parser. Extract search keywords from the user query and detect intent.

User query: "{}"

Respond ONLY with a JSON object, no explanation, no text before or after:
{{
  "keywords": ["word1", "word2"],
  "want_latest": true or false
}}

Rules:
- keywords: meaningful words to search in file names (lowercase, no accents needed, no stop words like "le/la/les/un/de/du/mon/ma/cherche/trouve/fichier")
- want_latest: true only if the user asks for the most recent/latest file ("dernier", "dernière", "récent", "nouveau", "latest", "most recent")
- Split contractions: "d'imposition" → "imposition", "l'EDF" → "EDF"

Output ONLY the JSON object."#,
        query
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_default();

    let response = client
        .post("http://127.0.0.1:11434/api/generate")
        .json(&serde_json::json!({
            "model": "llama3",
            "prompt": prompt,
            "stream": false,
            "options": { "temperature": 0.0 }
        }))
        .send()
        .await;

    // En cas d'échec → fallback : parser local simple
    let Ok(response) = response else {
        println!("Ollama inaccessible, fallback local");
        return local_parse(query);
    };

    let json: serde_json::Value = match response.json().await {
        Ok(v) => v,
        Err(_) => return local_parse(query),
    };

    let raw = json["response"].as_str().unwrap_or("");
    println!("Ollama parse réponse : {}", raw);

    // Extraction robuste : cherche { ... } même si Ollama bavarde
    let json_str = match (raw.find('{'), raw.rfind('}')) {
        (Some(start), Some(end)) if end > start => &raw[start..=end],
        _ => {
            println!("Pas de JSON dans la réponse, fallback local");
            return local_parse(query);
        }
    };

    match serde_json::from_str::<ParsedQuery>(json_str) {
        Ok(parsed) => {
            println!("Ollama parsed : {:?}", parsed);
            parsed
        }
        Err(e) => {
            println!("Erreur parsing Ollama : {:?}, fallback local", e);
            local_parse(query)
        }
    }
}

// =========================================================
// FALLBACK LOCAL : parser de requête sans Ollama
// =========================================================
fn local_parse(query: &str) -> ParsedQuery {

    let stop_words = [
        "cherche", "chercher", "trouve", "trouver", "montre", "affiche",
        "le", "la", "les", "l", "un", "une", "des", "du", "de", "d",
        "mon", "ma", "mes", "moi", "fichier", "fichiers", "dossier",
        "dossiers", "document", "documents",
    ];

    let recency_words = [
        "dernier", "dernière", "récent", "récente",
        "nouveau", "nouvelle", "récents", "récentes",
    ];

    let q_lower = query.to_lowercase();

    let want_latest = recency_words.iter()
        .any(|w| q_lower.split_whitespace().any(|t| t == *w));

    let keywords: Vec<String> = q_lower
        .split_whitespace()
        .filter(|w| !stop_words.contains(w) && !recency_words.contains(w))
        .flat_map(|w| w.split('\'').filter(|p| !p.is_empty() && !stop_words.contains(p)))
        .map(|s| s.to_string())
        .collect();

    println!("Fallback local parse : {:?} | want_latest: {}", keywords, want_latest);

    ParsedQuery { keywords, want_latest }
}

// =========================================================
// COMMANDE TAURI : SEARCH FILES
// =========================================================
#[tauri::command]
async fn search_files(query: String, paths: Vec<String>) -> SearchResult {

    println!("Recherche : {}", query);

    // 1. Ollama parse l'intention (avec fallback local)
    let parsed = ollama_parse(&query).await;

    println!("Intent : {:?}", parsed);

    // 2. Scan disque dans un thread dédié
    let results = tokio::task::spawn_blocking(move || {
        scan_files(&parsed.keywords, parsed.want_latest, &paths)
    }).await.unwrap_or_default();

    println!("Résultats : {}", results.len());

    let reply = if results.is_empty() {
        format!("Aucun résultat trouvé pour : {}", query)
    } else {
        format!("{} résultat(s) trouvé(s)", results.len())
    };

    SearchResult { reply, files: results }
}

// =========================================================
// HELPERS SCORING
// =========================================================

// Extrait la première année plausible (1900-2099) d'une chaîne.
// "avis d'imposition 2025.pdf" → 2025
// "travis_2019_backup" → 2019
// Retourne 0 si aucune année trouvée.
fn extract_year(s: &str) -> u32 {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 3 < bytes.len() {
        // Cherche 4 chiffres consécutifs
        if bytes[i..i+4].iter().all(|b| b.is_ascii_digit()) {
            let year: u32 = s[i..i+4].parse().unwrap_or(0);
            if year >= 1900 && year <= 2099 {
                // Vérifie que ce n'est pas au milieu d'un nombre plus long
                let before_ok = i == 0 || !bytes[i-1].is_ascii_digit();
                let after_ok  = i + 4 >= bytes.len() || !bytes[i+4].is_ascii_digit();
                if before_ok && after_ok {
                    return year;
                }
            }
        }
        i += 1;
    }
    0
}

// Vérifie que `needle` apparaît comme mot entier dans `haystack`.
// Séparateurs : tout caractère non alphanumérique (espace, _, -, ., apostrophe…)
// Exemples :
//   "travis_gh_page" contient "avis" ? NON  (précédé de "tr")
//   "avis d imposition" contient "avis" ? OUI (début de chaîne)
//   "mon-avis.pdf"     contient "avis" ? OUI (précédé de "-")
fn contains_whole_word(haystack: &str, needle: &str) -> bool {
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    let nlen = n.len();
    if nlen == 0 || nlen > h.len() { return false; }

    for i in 0..=(h.len() - nlen) {
        if &h[i..i + nlen] == n {
            let before_ok = i == 0 || !h[i - 1].is_ascii_alphanumeric();
            let after_ok  = i + nlen == h.len() || !h[i + nlen].is_ascii_alphanumeric();
            if before_ok && after_ok {
                return true;
            }
        }
    }
    false
}

// =========================================================
// SCAN FICHIERS
// =========================================================
fn scan_files(keywords: &[String], want_latest: bool, paths: &[String]) -> Vec<FileItem> {

    let mut results: Vec<FileItem> = vec![];

    for base in paths {
        let base_path = Path::new(base);

        if !base_path.exists() {
            continue;
        }

        println!("Scan : {}", base);

        for entry in WalkDir::new(base_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            let full_path = path.display().to_string().to_lowercase();

            let mut score = 0;
            let mut matched_count = 0;

            // =====================================================
            // MATCHING DES MOTS-CLÉS
            // =====================================================
            //
            // - Nom du fichier = bonus fort
            // - Chemin du fichier = bonus faible
            // - Tous les mots ne sont plus obligatoires
            //
            for kw in keywords {

                let in_name = contains_whole_word(&file_name, kw);
                let in_path = contains_whole_word(&full_path, kw);

                if in_name || in_path {

                    matched_count += 1;

                    if in_name {
                        score += 20;
                    }

                    if in_path {
                        score += 10;
                    }
                }
            }

            // =====================================================
            // FILTRE MINIMUM
            // =====================================================
            //
            // Aucun mot-clé trouvé => rejet
            //
            if matched_count == 0 {
                continue;
            }

            // =====================================================
            // SOUPLESSE CONTRÔLÉE
            // =====================================================
            //
            // Exemple :
            //
            // 1 mot demandé  -> 1 mot requis
            // 2 mots demandés -> 1 mot requis
            // 3 mots demandés -> 2 mots requis
            // 4 mots demandés -> 2 mots requis
            // 5 mots demandés -> 3 mots requis
            //
            let min_required =
                ((keywords.len() as f32) * 0.5).ceil() as usize;

            if matched_count < min_required {
                continue;
            }

            // =====================================================
            // BONUS SI TOUS LES MOTS SONT TROUVÉS
            // =====================================================
            //
            // Permet de favoriser :
            //
            // "avis imposition"
            //
            // devant :
            //
            // "avis impot"
            //
            if matched_count == keywords.len() {
                score += 50;
            }

            // =====================================================
            // MÉTADONNÉES
            // =====================================================

            let file_type = if path.is_dir() {
                "folder".to_string()
            } else {
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("file")
                    .to_string()
            };

            let modified_secs = entry.metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let year_in_name = extract_year(&file_name);

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

                year_in_name,
            });
        }
    }

    // =========================================================
    // TRI DES RÉSULTATS
    // =========================================================
    //
    // 1. Pertinence (score)
    // 2. Année dans le nom
    // 3. Date de modification
    //
    if want_latest {

        results.sort_by(|a, b| {

            b.score.cmp(&a.score)

                .then_with(|| {
                    b.modified_secs.cmp(&a.modified_secs)
                })

                .then_with(|| {
                    b.year_in_name.cmp(&a.year_in_name)
                })
        });

    } else {

        results.sort_by(|a, b| {

            b.score.cmp(&a.score)

                .then_with(|| {
                    b.year_in_name.cmp(&a.year_in_name)
                })

                .then_with(|| {
                    b.modified_secs.cmp(&a.modified_secs)
                })
        });

    }

    // =========================================================
    // LIMITATION DES RÉSULTATS
    // =========================================================
    //
    // Quand l'utilisateur demande "le dernier",
    // on affiche quand même quelques propositions
    // plutôt qu'un seul résultat potentiellement faux.
    //
    let limit = if want_latest { 5 } else { 50 };

    results.into_iter().take(limit).collect()
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
