import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useStore } from "../../stores/useStore";
import { Search, Download, Sun, Moon, Home } from "lucide-react";

export function Toolbar({ onSearch }: { onSearch: () => void }) {
  const { analysis, theme, toggleTheme, reset, setShowExportDialog } = useStore();

  const handleExport = () => {
    setShowExportDialog(true);
  };

  const handleHome = () => {
    reset();
  };

  return (
    <div className="flex items-center gap-2">
      <button
        onClick={onSearch}
        className="btn btn-secondary"
        title="Search (Ctrl+F)"
      >
        <Search size={15} />
        <span className="hidden sm:inline">Search</span>
      </button>

      <button
        onClick={handleExport}
        className="btn btn-secondary"
        title="Export Report"
      >
        <Download size={15} />
        <span className="hidden sm:inline">Export</span>
      </button>

      <div className="w-px h-6" style={{ backgroundColor: "var(--border-color)" }} />

      <button onClick={toggleTheme} className="p-2 rounded-lg transition-colors" style={{ color: "var(--text-secondary)" }} title="Toggle theme">
        {theme === "light" ? <Moon size={16} /> : <Sun size={16} />}
      </button>

      <button onClick={handleHome} className="p-2 rounded-lg transition-colors" style={{ color: "var(--text-secondary)" }} title="Open another APK">
        <Home size={16} />
      </button>
    </div>
  );
}
