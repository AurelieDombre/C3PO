import { useState, useRef, useEffect } from "react";
import "./assets/css/App.css"

export default function App() {
  const [messages, setMessages] = useState([]);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(false);
  const bottomRef = useRef(null);
  const inputRef = useRef(null);
 
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, loading]);
 
  async function handleSearch() {
    const text = query.trim();
    if (!text || loading) return;
 
    const newMessages = [...messages, { role: "user", content: text }];
    setMessages(newMessages);
    setQuery("");
    setLoading(true);
 
    try {
      const response = await fetch("http://127.0.0.1:8000/chat", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ messages: newMessages }),
      });
      const data = await response.json();
      setMessages([...newMessages, { role: "assistant", content: data.reply, files: data.files }]);
    } catch {
      setMessages([
        ...newMessages,
        { role: "assistant", content: "⚠️ Impossible de joindre le serveur local." },
      ]);
    } finally {
      setLoading(false);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }
 
  return (

      <div className="ai-root">
        <div className="ai-shell">
 
          {/* Header */}
          <div className="ai-header">
            <div className="ai-header-dot" />
            <span className="ai-header-title">Assistant IA Local</span>
            <span className="ai-header-sub">ollama · local</span>
          </div>
 
          {/* Messages */}
          <div className="ai-messages">
            {messages.length === 0 && (
              <div className="ai-empty">
                <div className="ai-empty-icon">◈</div>
                <span className="ai-empty-label">En attente d'un message…</span>
              </div>
            )}
 
            {messages.map((msg, i) => (
              <div key={i} className={`ai-msg ai-msg--${msg.role}`}>
                <span className="ai-msg-label">
                  {msg.role === "user" ? "Vous" : "Assistant"}
                </span>
                <div className="ai-bubble">{msg.content}</div>
                {/* Fichiers retournés par la recherche */}
                {msg.files && msg.files.length > 0 && (
                  <div className="ai-files">
                    {msg.files.map((path, j) => (
                      <div key={j} className="ai-file-item">
                        <span className="ai-file-icon">📄</span>
                        <span className="ai-file-path">{path}</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ))}
 
            {loading && (
              <div className="ai-msg ai-msg--assistant">
                <span className="ai-msg-label">Assistant</span>
                <div className="ai-typing">
                  <span /><span /><span />
                </div>
              </div>
            )}
 
            <div ref={bottomRef} />
          </div>
 
          {/* Input */}
          <div className="ai-footer">
            <div className="ai-input-row">
              <input
                ref={inputRef}
                className="ai-input"
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Cherche mes PDF de factures…"
                disabled={loading}
                onKeyDown={(e) => e.key === "Enter" && handleSearch()}
              />
              <button
                className="ai-send"
                onClick={handleSearch}
                disabled={loading || !query.trim()}
                aria-label="Envoyer"
              >
                ↑
              </button>
            </div>
            <p className="ai-hint">Entrée pour envoyer · Powered by Ollama</p>
          </div>
 
        </div>
      </div>

  );
}