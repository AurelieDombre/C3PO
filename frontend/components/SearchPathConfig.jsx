import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";

// Clé unique utilisée partout pour les chemins de recherche
const STORAGE_KEY = "search_paths";

export default function SearchPathConfig() {

  const [paths, setPaths] = useState(() => {
    // Lecture avec la même clé que l'écriture
    const saved = localStorage.getItem(STORAGE_KEY);
    return saved ? JSON.parse(saved) : [];
  });

  async function addFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected) return;

    const updated = [...paths, selected];
    setPaths(updated);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
  }

  function removeFolder(index) {
    const updated = paths.filter((_, i) => i !== index);
    setPaths(updated);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
  }

  // ⚠️  Le return est ici, à l'intérieur du composant
  //     mais EN DEHORS de addFolder()
  return (
    <div>
      <button onClick={addFolder}>
        Ajouter un dossier
      </button>
      <ul>
        {paths.map((p, i) => (
          <li key={i}>
            {p}
            <button onClick={() => removeFolder(i)} style={{ marginLeft: 8 }}>
              ✕
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
