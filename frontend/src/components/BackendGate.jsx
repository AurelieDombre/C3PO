import useBackendStatus from "../hooks/useBackendStatus.jsx";

export default function BackendGate({ children }) {
  const ready = useBackendStatus();

  if (!ready) {
    return (
      <div style={{ padding: 20 }}>
        <h2>Initialisation backend...</h2>
      </div>
    );
  }

  return children;
}