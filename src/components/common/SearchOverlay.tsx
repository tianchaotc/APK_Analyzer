import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../../stores/useStore";
import { Search, X } from "lucide-react";
import type { SearchResult } from "../../types";

export function SearchOverlay({ onClose }: { onClose: () => void }) {
  const { setSearchQuery, setSearchResults } = useStore();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);

  const handleSearch = useCallback(async (q: string) => {
    if (q.trim().length < 2) {
      setResults([]);
      return;
    }
    setLoading(true);
    try {
      const res = await invoke<SearchResult[]>("search_global", { query: q });
      setResults(res);
      setSearchQuery(q);
      setSearchResults(res);
    } catch {
      setResults([]);
    } finally {
      setLoading(false);
    }
  }, [setSearchQuery, setSearchResults]);

  useEffect(() => {
    const timer = setTimeout(() => handleSearch(query), 200);
    return () => clearTimeout(timer);
  }, [query, handleSearch]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [onClose]);

  return (
    <div
      className="fixed inset-0 flex items-start justify-center pt-20 z-50"
      style={{ backgroundColor: "rgba(0,0,0,0.4)" }}
      onClick={onClose}
    >
      <div
        className="w-full max-w-2xl rounded-xl shadow-2xl overflow-hidden"
        style={{ backgroundColor: "var(--bg-primary)" }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search input */}
        <div className="flex items-center gap-3 px-4 py-3 border-b" style={{ borderColor: "var(--border-color)" }}>
          <Search size={18} style={{ color: "var(--text-tertiary)" }} />
          <input
            autoFocus
            type="text"
            placeholder="Search across manifest, permissions, resources, DEX, libraries, certificates..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="flex-1 bg-transparent outline-none text-sm"
            style={{ color: "var(--text-primary)" }}
          />
          <button onClick={onClose} className="p-1 rounded hover:bg-opacity-80" style={{ color: "var(--text-tertiary)" }}>
            <X size={16} />
          </button>
        </div>

        {/* Results */}
        <div className="max-h-96 overflow-y-auto">
          {loading && (
            <div className="px-4 py-3 text-sm" style={{ color: "var(--text-tertiary)" }}>Searching...</div>
          )}
          {!loading && query.length >= 2 && results.length === 0 && (
            <div className="px-4 py-3 text-sm" style={{ color: "var(--text-tertiary)" }}>No results found</div>
          )}
          {!loading && results.length > 0 && (
            <div className="py-1">
              {results.map((r, i) => (
                <div
                  key={i}
                  className="px-4 py-2.5 hover:bg-opacity-80 cursor-pointer"
                  style={{ borderBottom: "1px solid var(--border-subtle)" }}
                  onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = "var(--bg-hover)")}
                  onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
                >
                  <div className="flex items-center justify-between mb-0.5">
                    <span className="text-sm font-medium" style={{ color: "var(--text-primary)" }}>{r.title}</span>
                    <span className="text-xs badge badge-neutral">{r.category}</span>
                  </div>
                  <p className="text-xs" style={{ color: "var(--text-tertiary)" }}>{r.detail}</p>
                </div>
              ))}
            </div>
          )}
          {!loading && query.length < 2 && (
            <div className="px-4 py-8 text-center">
              <p className="text-sm" style={{ color: "var(--text-tertiary)" }}>Type at least 2 characters to search</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
