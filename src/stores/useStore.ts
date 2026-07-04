import { create } from "zustand";
import type { ApkAnalysis, ProgressUpdate, NavSection, RecentFile } from "../types";

interface AppState {
  // Current analysis
  analysis: ApkAnalysis | null;
  isAnalyzing: boolean;
  progress: ProgressUpdate | null;
  error: string | null;

  // Navigation
  activeSection: NavSection;

  // Recent files
  recentFiles: RecentFile[];

  // Search
  searchQuery: string;
  searchResults: any[];

  // Theme
  theme: "light" | "dark";

  // Export
  showExportDialog: boolean;

  // Actions
  setAnalysis: (a: ApkAnalysis | null) => void;
  setAnalyzing: (v: boolean) => void;
  setProgress: (p: ProgressUpdate | null) => void;
  setError: (e: string | null) => void;
  setActiveSection: (s: NavSection) => void;
  setRecentFiles: (f: RecentFile[]) => void;
  setSearchQuery: (q: string) => void;
  setSearchResults: (r: any[]) => void;
  toggleTheme: () => void;
  setShowExportDialog: (v: boolean) => void;
  reset: () => void;
}

export const useStore = create<AppState>((set) => ({
  analysis: null,
  isAnalyzing: false,
  progress: null,
  error: null,
  activeSection: "overview",
  recentFiles: [],
  searchQuery: "",
  searchResults: [],
  theme: (localStorage.getItem("theme") as "light" | "dark") || "light",
  showExportDialog: false,

  setAnalysis: (a) => set({ analysis: a }),
  setAnalyzing: (v) => set({ isAnalyzing: v }),
  setProgress: (p) => set({ progress: p }),
  setError: (e) => set({ error: e }),
  setActiveSection: (s) => set({ activeSection: s }),
  setRecentFiles: (f) => set({ recentFiles: f }),
  setSearchQuery: (q) => set({ searchQuery: q }),
  setSearchResults: (r) => set({ searchResults: r }),
  toggleTheme: () =>
    set((state) => {
      const newTheme = state.theme === "light" ? "dark" : "light";
      localStorage.setItem("theme", newTheme);
      document.documentElement.classList.toggle("dark", newTheme === "dark");
      return { theme: newTheme };
    }),
  setShowExportDialog: (v) => set({ showExportDialog: v }),
  reset: () =>
    set({
      analysis: null,
      progress: null,
      error: null,
      activeSection: "overview",
      searchQuery: "",
      searchResults: [],
    }),
}));
