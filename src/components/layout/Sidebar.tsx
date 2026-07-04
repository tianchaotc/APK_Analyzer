import { useStore } from "../../stores/useStore";
import type { NavSection } from "../../types";
import {
  LayoutDashboard, FileCode, ShieldCheck, Boxes, Image,
  Cpu, Binary, Award, AlertTriangle, Sparkles
} from "lucide-react";

interface NavItem {
  id: NavSection;
  label: string;
  icon: React.ReactNode;
}

const navItems: NavItem[] = [
  { id: "overview", label: "Overview", icon: <LayoutDashboard size={17} /> },
  { id: "manifest", label: "Manifest", icon: <FileCode size={17} /> },
  { id: "permissions", label: "Permissions", icon: <ShieldCheck size={17} /> },
  { id: "components", label: "Components", icon: <Boxes size={17} /> },
  { id: "resources", label: "Resources", icon: <Image size={17} /> },
  { id: "native_libs", label: "Native Libraries", icon: <Cpu size={17} /> },
  { id: "dex", label: "DEX", icon: <Binary size={17} /> },
  { id: "certificate", label: "Certificate", icon: <Award size={17} /> },
  { id: "security", label: "Security", icon: <AlertTriangle size={17} /> },
  { id: "ai_summary", label: "AI Summary", icon: <Sparkles size={17} /> },
];

export function Sidebar() {
  const { activeSection, setActiveSection, analysis } = useStore();

  const getBadge = (section: NavSection): number | null => {
    if (!analysis) return null;
    switch (section) {
      case "permissions": return analysis.permissions.summary.total || null;
      case "components": return analysis.components.stats.activities + analysis.components.stats.services + analysis.components.stats.receivers + analysis.components.stats.providers || null;
      case "native_libs": return analysis.native_libs.summary.total || null;
      case "dex": return analysis.dex.summary.total_dex_files || null;
      case "security": return analysis.security.issues.filter(i => i.severity === "critical" || i.severity === "high").length || null;
      default: return null;
    }
  };

  return (
    <div className="w-52 flex-shrink-0 flex flex-col py-4 px-2 border-r overflow-y-auto" style={{ borderColor: "var(--border-color)", backgroundColor: "var(--bg-secondary)" }}>
      <nav className="flex-1 space-y-0.5">
        {navItems.map((item) => (
          <div
            key={item.id}
            className={`nav-item ${activeSection === item.id ? "active" : ""}`}
            onClick={() => setActiveSection(item.id)}
          >
            {item.icon}
            <span className="flex-1">{item.label}</span>
            {getBadge(item.id) !== null && (
              <span className="badge badge-neutral">{getBadge(item.id)}</span>
            )}
          </div>
        ))}
      </nav>
    </div>
  );
}
