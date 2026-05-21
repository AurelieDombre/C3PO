import { useState } from "react";

export default function App() {

  const [files, setFiles] = useState([]);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(false);

  async function handleSearch() {

    setLoading(true);

    const response = await fetch(
      `http://127.0.0.1:8000/search?query=${query}`
    );

    const data = await response.json();

    setFiles(data.files);

    setLoading(false);
  }

  return (
    <div>

      <h1>Résultats</h1>

      <input
        type="text"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="Ex: cherche mes pdf de facture"
      />

      <button onClick={handleSearch}>
        Rechercher
      </button>

      {loading && <p>Recherche en cours...</p>}

      <hr />

      {files.map((file, index) => (
        <p key={index}>{file}</p>
      ))}

    </div>
  );
}