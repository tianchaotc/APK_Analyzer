import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useStore } from "../../stores/useStore";
import { FileJson, FileText, FileCode, FileSpreadsheet, X, Download, Loader2, Check } from "lucide-react";

export function ExportDialog() {
  const { showExportDialog, setShowExportDialog } = useStore();
  const [exporting, setExporting] = useState(false);
  const [success, setSuccess] = useState(false);

  if (!showExportDialog) return null;

  const formats = [
    { id: "json", label: "JSON", desc: "Structured data format", icon: <FileJson size={24} /> },
    { id: "markdown", label: "Markdown", desc: "Human-readable report", icon: <FileText size={24} /> },
    { id: "html", label: "HTML", desc: "Styled web report", icon: <FileCode size={24} /> },
    { id: "csv", label: "CSV", desc: "Spreadsheet format", icon: <FileSpreadsheet size={24} /> },
  ];

  const handleExport = async (format: string) => {
    setExporting(true);
    setSuccess(false);
    try {
      const extension = format;
      const filePath = await save({
        filters: [{ name: format.toUpperCase(), extensions: [extension] }],
        defaultPath: `apk-report.${extension}`,
      });
      if (filePath) {
        await invoke("export_report", { format, outputPath: filePath });
        setSuccess(true);
        setTimeout(() => {
          setShowExportDialog(false);
          setSuccess(false);
        }, 1500);
      }
    } catch (e) {
      console.error("Export failed:", e);
    } finally {
      setExporting(false);
    }
  };

  return (
    <div
      className="fixed inset-0 flex items-center justify-center z-50"
      style={{ backgroundColor: "rgba(0,0,0,0.4)" }}
      onClick={() => !exporting && setShowExportDialog(false)}
    >
      <div
        className="w-full max-w-md rounded-xl shadow-2xl overflow-hidden"
        style={{ backgroundColor: "var(--bg-primary)" }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b" style={{ borderColor: "var(--border-color)" }}>
          <h2 className="text-base font-semibold" style={{ color: "var(--text-primary)" }}>Export Report</h2>
          <button onClick={() => !exporting && setShowExportDialog(false)} className="p-1 rounded hover:bg-opacity-80" style={{ color: "var(--text-tertiary)" }}>
            <X size={18} />
          </button>
        </div>

        {/* Content */}
        <div className="p-5">
          {success ? (
            <div className="flex flex-col items-center py-8">
              <div className="w-12 h-12 rounded-full flex items-center justify-center mb-3" style={{ backgroundColor: "rgba(22,163,74,0.1)" }}>
                <Check size={24} style={{ color: "var(--success)" }} />
              </div>
              <p className="text-sm font-medium" style={{ color: "var(--text-primary)" }}>Export successful!</p>
            </div>
          ) : (
            <div className="grid grid-cols-2 gap-3">
              {formats.map((f) => (
                <button
                  key={f.id}
                  onClick={() => handleExport(f.id)}
                  disabled={exporting}
                  className="flex flex-col items-center gap-2 p-4 rounded-xl border transition-all hover:bg-opacity-80 disabled:opacity-50"
                  style={{
                    borderColor: "var(--border-color)",
                    backgroundColor: "var(--bg-secondary)",
                  }}
                >
                  <div style={{ color: "var(--accent)" }}>{f.icon}</div>
                  <span className="text-sm font-medium" style={{ color: "var(--text-primary)" }}>{f.label}</span>
                  <span className="text-xs text-center" style={{ color: "var(--text-tertiary)" }}>{f.desc}</span>
                </button>
              ))}
            </div>
          )}

          {exporting && (
            <div className="flex items-center justify-center gap-2 mt-4">
              <Loader2 size={16} className="animate-spin" style={{ color: "var(--accent)" }} />
              <span className="text-sm" style={{ color: "var(--text-secondary)" }}>Exporting...</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
