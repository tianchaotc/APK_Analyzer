import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../../stores/useStore";
import { Sidebar } from "./Sidebar";
import { Toolbar } from "./Toolbar";
import { ProgressBar } from "./ProgressBar";
import { OverviewPage } from "../../pages/OverviewPage";
import { ManifestPage } from "../../pages/ManifestPage";
import { PermissionsPage } from "../../pages/PermissionsPage";
import { ComponentsPage } from "../../pages/ComponentsPage";
import { ResourcesPage } from "../../pages/ResourcesPage";
import { NativeLibsPage } from "../../pages/NativeLibsPage";
import { DexPage } from "../../pages/DexPage";
import { CertificatePage } from "../../pages/CertificatePage";
import { SecurityPage } from "../../pages/SecurityPage";
import { AISummaryPage } from "../../pages/AISummaryPage";
import { SearchOverlay } from "../common/SearchOverlay";
import { Package, X } from "lucide-react";

export function MainLayout() {
  const { analysis, activeSection, isAnalyzing } = useStore();
  const [showSearch, setShowSearch] = useState(false);

  if (!analysis) return null;

  const renderPage = () => {
    switch (activeSection) {
      case "overview": return <OverviewPage />;
      case "manifest": return <ManifestPage />;
      case "permissions": return <PermissionsPage />;
      case "components": return <ComponentsPage />;
      case "resources": return <ResourcesPage />;
      case "native_libs": return <NativeLibsPage />;
      case "dex": return <DexPage />;
      case "certificate": return <CertificatePage />;
      case "security": return <SecurityPage />;
      case "ai_summary": return <AISummaryPage />;
      default: return <OverviewPage />;
    }
  };

  return (
    <div className="h-full flex flex-col">
      {/* Top bar */}
      <div className="flex items-center justify-between px-4 py-3 border-b" style={{ borderColor: "var(--border-color)", backgroundColor: "var(--bg-secondary)" }}>
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg flex items-center justify-center" style={{ backgroundColor: "var(--accent)" }}>
            <Package size={18} color="white" />
          </div>
          <div className="flex flex-col">
            <span className="text-sm font-semibold" style={{ color: "var(--text-primary)" }}>{analysis.overview.app_name || analysis.file_name}</span>
            <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>{analysis.overview.package_name}</span>
          </div>
        </div>
        <Toolbar onSearch={() => setShowSearch(true)} />
      </div>

      {/* Main content */}
      <div className="flex-1 flex overflow-hidden">
        <Sidebar />
        <div className="flex-1 overflow-auto p-6">
          <div className="fade-in" key={activeSection}>
            {renderPage()}
          </div>
        </div>
      </div>

      {/* Bottom progress bar */}
      {isAnalyzing && <ProgressBar />}

      {/* Search overlay */}
      {showSearch && <SearchOverlay onClose={() => setShowSearch(false)} />}
    </div>
  );
}
