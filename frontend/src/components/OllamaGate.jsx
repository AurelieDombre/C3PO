// src/components/OllamaGate.jsx

import { open } from "@tauri-apps/plugin-shell";
import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect, useRef } from "react";

export default function OllamaGate({ children, ollama = {} }) {

    const { loading, installed, running, models, refresh } = ollama;
    const [pulling, setPulling] = useState(false);
    const [pullSeconds, setPullSeconds] = useState(0);
    const timerRef = useRef(null);

    // Compteur de secondes affiché pendant le pull
    useEffect(() => {
        if (pulling) {
            setPullSeconds(0);
            timerRef.current = setInterval(() => {
                setPullSeconds(s => s + 1);
            }, 1000);
        } else {
            clearInterval(timerRef.current);
        }
        return () => clearInterval(timerRef.current);
    }, [pulling]);

    async function startOllama() {
        try {
            await invoke("start_ollama");
            // Laisse 2 s à Ollama pour démarrer avant de re-checker
            setTimeout(() => refresh?.(), 2000);
        } catch (err) {
            console.error("start_ollama error:", err);
        }
    }

    async function pullModel() {
        setPulling(true);
        try {
            // Lance le pull (retourne immédiatement côté Rust)
            await invoke("pull_model", { model: "llama3" });

            // Poll toutes les 5 s jusqu'à ce que le modèle apparaisse
            const interval = setInterval(async () => {
                await refresh?.();
                // refresh met à jour ollama.models dans le hook parent ;
                // si un modèle est dispo on arrête le polling
                if (models && models.length > 0) {
                    clearInterval(interval);
                    setPulling(false);
                }
            }, 5000);

            // Sécurité : arrêt forcé après 15 min quoi qu'il arrive
            setTimeout(() => {
                clearInterval(interval);
                setPulling(false);
                refresh?.();
            }, 15 * 60 * 1000);

        } catch (err) {
            console.error("pull_model error:", err);
            setPulling(false);
        }
    }

    // ── Écrans de garde ──────────────────────────────────────

    if (loading) {
        return <div className="ai-gate">Vérification d'Ollama…</div>;
    }

    if (!installed) {
        return (
            <div className="ai-gate">
                <h2>📦 Ollama non installé</h2>
                <p>Ollama est requis pour faire tourner l'IA localement.</p>
                <button onClick={() => open("https://ollama.com/download")}>
                    Installer Ollama
                </button>
                <button onClick={refresh} style={{ marginLeft: 8 }}>
                    J'ai installé, réessayer
                </button>
            </div>
        );
    }

    if (!running) {
        return (
            <div className="ai-gate">
                <h2>⚠️ Ollama arrêté</h2>
                <p>Le serveur Ollama ne répond pas.</p>
                <button onClick={startOllama}>Démarrer Ollama</button>
                <button onClick={refresh} style={{ marginLeft: 8 }}>Réessayer</button>
            </div>
        );
    }

    if (!models || models.length === 0) {
        return (
            <div className="ai-gate">
                <h2>🧠 Aucun modèle installé</h2>
                <p>Ollama tourne mais aucun modèle n'est disponible.</p>
                <button onClick={pullModel} disabled={pulling}>
                    {pulling ? "Téléchargement en cours…" : "Installer llama3"}
                </button>
                <button onClick={refresh} disabled={pulling} style={{ marginLeft: 8 }}>
                    Actualiser
                </button>
                {pulling && (
                    <p className="ai-gate-hint">
                        Téléchargement en cours ({pullSeconds}s)…
                        Le modèle fait ~4 Go, cela peut prendre plusieurs minutes.
                    </p>
                )}
            </div>
        );
    }

    return children;
}
