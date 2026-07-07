import { useEffect } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useStore } from "./stores/useStore";
import { HomePage } from "./pages/HomePage";
import { MainLayout } from "./components/layout/MainLayout";
import { ExportDialog } from "./components/common/ExportDialog";

export default function App() {
  const { analysis, theme, isAnalyzing, setProgress, setRecentFiles } = useStore();

  // Apply theme on mount
  useEffect(() => {
    document.documentElement.classList.toggle("dark", theme === "dark");
  }, [theme]);

  // Load recent files on mount
  useEffect(() => {
    if (!isTauri()) return;

    invoke<import("./types").RecentFile[]>("get_recent_files")
      .then((files) => setRecentFiles(files))
      .catch(() => {});
  }, []);

  // Listen for progress events
  useEffect(() => {
    if (!isTauri()) return;

    const unlisten = listen<import("./types").ProgressUpdate>("analysis-progress", (event) => {
      setProgress(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <>
      {analysis || isAnalyzing ? (
        <>
          <MainLayout />
          <ExportDialog />
        </>
      ) : (
        <HomePage />
      )}
    </>
  );
}
