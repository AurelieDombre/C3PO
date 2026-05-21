import { useEffect, useState } from "react";
import "./App.css";

function App() {

  const [files, setFiles] = useState([]);

  useEffect(() => {

    async function fetchFiles() {

      const response = await fetch(
        "http://127.0.0.1:8000/search?extension=pdf"
      );

      const data = await response.json();

      console.log(data);

      setFiles(data.files);
    }

    fetchFiles();

  }, []);

  return (
    <div>

      <h1>Résultats</h1>

      {files.map((file, index) => (
        <p key={index}>{file}</p>
      ))}

    </div>
  );
}

export default App;