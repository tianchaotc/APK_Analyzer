import { useCallback, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useStore } from "../stores/useStore";
import { FileSearch, Clock, X, Package, Sun, Moon, Upload, Loader2 } from "lucide-react";
import { formatFileSize } from "../utils/format";

export function HomePage() {
  const { setAnalysis, setAnalyzing, setError, recentFiles, setRecentFiles, theme, toggleTheme, error, isAnalyzing, progress } = useStore();
  const [isDragging, setIsDragging] = useState(false);
  const dragCounter = useRef(0);

  const handleAnalyze = useCallback(async (path: string) => {
    setAnalyzing(true);
    setError(null);
    try {
      const result = await invoke<import("../types").ApkAnalysis>("analyze_apk", { path });
      setAnalysis(result);
    } catch (e: any) {
      setError(typeof e === "string" ? e : e.message || "Analysis failed");
    } finally {
      setAnalyzing(false);
    }
  }, [setAnalysis, setAnalyzing, setError]);

  const handleBrowse = useCallback(async () => {
    try {
      const selected = await open({
        filters: [{ name: "APK files", extensions: ["apk"] }],
        multiple: false,
      });
      if (selected) {
        await handleAnalyze(selected as string);
      }
    } catch (e) {
      // Dialog cancelled
    }
  }, [handleAnalyze]);

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    dragCounter.current = 0;
    const files = e.dataTransfer.files;
    if (files.length > 0) {
      const file = files[0];
      // Tauri provides the path via the path property
      const path = (file as any).path || file.name;
      if (path && path.endsWith(".apk")) {
        await handleAnalyze(path);
      } else {
        setError("Please drop an APK file");
      }
    }
  }, [handleAnalyze, setError]);

  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    dragCounter.current++;
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    dragCounter.current--;
    if (dragCounter.current === 0) {
      setIsDragging(false);
    }
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
  }, []);

  const handleClearRecent = useCallback(async () => {
    await invoke("clear_recent_files");
    setRecentFiles([]);
  }, [setRecentFiles]);

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b" style={{ borderColor: "var(--border-color)" }}>
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-lg flex items-center justify-center" style={{ backgroundColor: "var(--accent)" }}>
            <Package size={20} color="white" />
          </div>
          <div>
            <h1 className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>APK Analyzer</h1>
            <p className="text-xs" style={{ color: "var(--text-tertiary)" }}>AI-Powered APK Inspection Tool</p>
          </div>
        </div>
        <button onClick={toggleTheme} className="p-2 rounded-lg hover:bg-opacity-80 transition-colors" style={{ color: "var(--text-secondary)" }}>
          {theme === "light" ? <Moon size={18} /> : <Sun size={18} />}
        </button>
      </div>

      {/* Main content */}
      <div className="flex-1 flex items-center justify-center p-8" onDrop={handleDrop} onDragOver={handleDragOver} onDragEnter={handleDragEnter} onDragLeave={handleDragLeave}>
        {isAnalyzing ? (
          <div className="text-center fade-in">
            <Loader2 size={64} className="animate-spin mx-auto mb-6" style={{ color: "var(--accent)" }} />
            <h2 className="text-xl font-semibold mb-2" style={{ color: "var(--text-primary)" }}>{progress?.stage || "Analyzing..."}</h2>
            <p className="text-sm mb-6" style={{ color: "var(--text-secondary)" }}>{progress?.message || "Processing APK file"}</p>
            <div className="w-80 h-2 rounded-full overflow-hidden mx-auto" style={{ backgroundColor: "var(--bg-tertiary)" }}>
              <div className="h-full rounded-full transition-all duration-300" style={{ width: `${progress?.percent || 0}%`, backgroundColor: "var(--accent)" }} />
            </div>
            <p className="text-xs mt-2" style={{ color: "var(--text-tertiary)" }}>{progress?.percent || 0}%</p>
          </div>
        ) : (
          <div className="w-full max-w-2xl">
            {/* Drop zone */}
            <div
              className={`rounded-2xl border-2 border-dashed p-12 text-center transition-all cursor-pointer ${isDragging ? "drag-active" : ""}`}
              style={{
                borderColor: isDragging ? "var(--accent)" : "var(--border-color)",
                backgroundColor: isDragging ? "var(--accent-subtle)" : "var(--bg-secondary)",
              }}
              onClick={handleBrowse}
            >
              <div className="w-20 h-20 rounded-2xl flex items-center justify-center mx-auto mb-6" style={{ backgroundColor: "var(--accent-subtle)" }}>
                <Upload size={36} style={{ color: "var(--accent)" }} />
              </div>
              <h2 className="text-2xl font-bold mb-2" style={{ color: "var(--text-primary)" }}>Drop APK Here</h2>
              <p className="text-sm mb-6" style={{ color: "var(--text-secondary)" }}>or click to browse</p>
              <div className="inline-flex items-center gap-2 px-5 py-2.5 rounded-lg text-sm font-medium text-white" style={{ backgroundColor: "var(--accent)" }}>
                <FileSearch size={16} />
                Browse APK
              </div>
            </div>

            {error && (
              <div className="mt-4 p-3 rounded-lg text-sm" style={{ backgroundColor: "rgba(220, 38, 38, 0.1)", color: "var(--danger)" }}>
                {error}
              </div>
            )}

            {/* Recent files */}
            {recentFiles.length > 0 && (
              <div className="mt-8">
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-2 text-sm font-medium" style={{ color: "var(--text-secondary)" }}>
                    <Clock size={15} />
                    Recent Files
                  </div>
                  <button onClick={handleClearRecent} className="text-xs hover:underline" style={{ color: "var(--text-tertiary)" }}>
                    Clear all
                  </button>
                </div>
                <div className="space-y-1.5">
                  {recentFiles.slice(0, 8).map((file) => (
                    <div
                      key={file.path}
                      className="flex items-center gap-3 px-4 py-3 rounded-lg cursor-pointer transition-colors hover:bg-opacity-80"
                      style={{ backgroundColor: "var(--bg-secondary)", border: "1px solid " + "var(--border-subtle)" }}
                      onClick={() => handleAnalyze(file.path)}
                    >
                      <Package size={18} style={{ color: "var(--accent)" }} />
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium truncate" style={{ color: "var(--text-primary)" }}>{file.name}</p>
                        <p className="text-xs truncate" style={{ color: "var(--text-tertiary)" }}>{file.path}</p>
                      </div>
                      <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>{formatFileSize(file.size)}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
