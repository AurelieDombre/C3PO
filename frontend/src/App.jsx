import { useState } from "react";

export default function App() {
  const [files, setFiles] = useState([]);
  const [extension, setExtension] = useState("pdf");
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(false);

  async function handleSearch() {

    setLoading(true);

    const response = await fetch(
      `http://127.0.0.1:8000/search?extension=${extension}&keyword=${query}`
    );

    const data = await response.json();
    setFiles(data.files);
    setLoading(false);


  }

  return (
    <div>
      <h1>Résultats</h1>



      {files.map((file, index) => (
        <p key={index}>{file}</p>
      ))}

      <div className="mt-6 flex max-w-md gap-x-4">
        <select
          value={extension}
          onChange={(e) => setExtension(e.target.value)}
        >
          <option value="pdf">PDF</option>
          <option value="odt">Libre Office</option>
        </select>

        <input
          placeholder="Pose ta question..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              handleSearch();
            }
          }}
        />

        <button
          onClick={handleSearch}
          className="bg-indigo-500 px-3.5 py-2.5 text-sm font-semibold text-white"
        >
          valider
        </button>


        {loading && <p>Recherche en cours...</p>}


      </div>
    </div>
  );
}