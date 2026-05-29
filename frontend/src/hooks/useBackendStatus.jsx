import { useEffect, useState } from "react";

export default function useBackendStatus() {
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let interval;

    async function check() {
      try {
        const res = await fetch("http://127.0.0.1:8000/health");
        if (res.ok) setReady(true);
      } catch {
        setReady(false);
      }
    }

    check();
    interval = setInterval(check, 2000);

    return () => clearInterval(interval);
  }, []);

  return ready;
}