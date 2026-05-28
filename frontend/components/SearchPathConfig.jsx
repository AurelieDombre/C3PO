import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";

export default function SearchPathConfig() {

  const [paths, setPaths] = useState(() => {
  const saved = localStorage.getItem("paths");

    return saved ? JSON.parse(saved) : [];
    });

  async function addFolder() {
    console.log("CLICK");

    const selected = await open({ directory: true });
    console.log("selected =", selected);
    //const selected = await open({
      //directory: true,
      //multiple: false,
    //});

    if (!selected) return;

    const updated = [...paths, selected];

    setPaths(updated);

    localStorage.setItem(
      "search_paths",
      JSON.stringify(updated)
    );
  }


  return (
    <div>

      <button onClick={addFolder}>
        Ajouter un dossier
      </button>

      <ul>
        {paths.map((p, i) => (
          <li key={i}>{p}</li>
        ))}
      </ul>

    </div>
  );
}