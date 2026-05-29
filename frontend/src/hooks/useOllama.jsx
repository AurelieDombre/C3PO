// src/hooks/useOllama.js

import { useEffect, useState } from "react";

export function useOllama() {

  const [status, setStatus] = useState({
    loading: true,
    available: false,
  });

  useEffect(() => {

    async function checkOllama() {

      try {

        const response = await fetch(
          "http://127.0.0.1:11434/api/tags"
        );

        setStatus({
          loading: false,
          available: response.ok,
        });

      } catch (err) {

        console.error("Ollama check failed:", err);

        setStatus({
          loading: false,
          available: false,
        });
      }
    }

    checkOllama();

  }, []);

  return status;
}