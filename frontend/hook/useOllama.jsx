// src/hooks/useOllama.js
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useOllama() {
  const [status, setStatus] = useState({
    loading: true,
    available: false
  });

  useEffect(() => {
    invoke("is_ollama_available")
      .then((res) => {
        setStatus({ loading: false, available: res });
      })
      .catch(() => {
        setStatus({ loading: false, available: false });
      });
  }, []);

  return status;
}