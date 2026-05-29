use serde::Serialize;

// Ce fichier contient la partie Rust de l'application Tauri.
// Rust joue ici le role de "backend local" : le front-end JavaScript appelle
// ces fonctions pour executer du code natif.

// `Serialize` permet de transformer ces structs en JSON pour les renvoyer au
// front-end.
#[derive(Serialize)]
struct SearchResult {
    // Message principal renvoye a l'interface.
    reply: String,
    // Liste des fichiers trouves. Pour l'instant elle est vide, car la
    // recherche n'est pas encore implemente.
    files: Vec<FileItem>,
}

#[derive(Serialize)]
struct FileItem {
    // Nom affiche dans l'UI.
    name: String,
    // Chemin complet du fichier sur le disque.
    path: String,
    // Type de fichier (pdf, txt, etc.).
    file_type: String,
}

// `#[tauri::command]` expose cette fonction au front-end.
// Cote JavaScript, elle sera appelee avec `invoke("search_files", ...)`.
#[tauri::command]
fn search_files(query: String, paths: Vec<String>) -> SearchResult {
    // `println!` ecrit dans la console du process Rust. C'est utile pour le
    // debug, mais ce message n'apparait pas directement dans l'UI.
    println!("Recherche : {}", query);
    println!("Paths : {:?}", paths);

    // Ici on renvoie une reponse factice.
    // `format!` construit une String.
    // `vec![]` cree un vecteur vide.
    SearchResult {
        reply: format!("Recherche reçue : {}", query),
        files: vec![],
    }
}

// Point d'entree de l'application Tauri.
// Sur mobile, cet attribut indique a Tauri quelle fonction utiliser comme
// entree.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // `Builder` construit l'application piece par piece.
    // Les plugins ajoutent des capacites supplementaires au runtime Tauri.
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        // On declare ici les commandes Rust visibles depuis le front-end.
        .invoke_handler(tauri::generate_handler![search_files])
        // Lance l'application avec la configuration generee par Tauri.
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
