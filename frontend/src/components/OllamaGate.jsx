// src/components/OllamaGate.jsx
import { open } from "@tauri-apps/plugin-shell";

export default function OllamaGate({ children, ollama = {} }) {
    const { loading = true, available = false } = ollama;

    if (loading) {
        return <div className="ai-loading">Vérification du moteur IA...</div>;
    }

    if (!available) {
        return (
            <div className="ai-gate">
                <h2>⚠️ Ollama n’est pas installé</h2>

                <button onClick={() => open("https://ollama.com/download")}>
                    Installer Ollama
                </button>
            </div>
        );
    }

    return children;
}