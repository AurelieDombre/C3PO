import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

export function useOllama() {

  const mounted = useRef(false);

  const [status, setStatus] = useState({
    loading: true,
    installed: false,
    running: false,
    models: []
  });

  async function refresh() {
    try {
      const result = await invoke("ollama_status");

      if (!mounted.current) return;

      setStatus({
        loading: false,
        ...result
      });

      }catch (err) {
        console.warn("Ollama not reachable:", err);

        setStatus({
          loading: false,
          installed: false,
          running: false,
          models: []
        });
      }
  }

  useEffect(() => {
    mounted.current = true;

    // ⚠️ important : on décale le premier call (pas immédiat sync)
    const init = setTimeout(() => {
      refresh();
    }, 0);

    const timer = setInterval(() => {
      refresh();
    }, 3000);

    return () => {
      mounted.current = false;
      clearTimeout(init);
      clearInterval(timer);
    };

  }, []);

  return {
    ...status,
    refresh
  };
}