import { useStore } from "../../stores/useStore";
import type { NavSection, PluginResult } from "../../types";
import { resolvePluginIcon } from "../../utils/pluginIcons";
import {
  LayoutDashboard, FileCode, ShieldCheck, Boxes, Image,
  Cpu, Binary, Award, AlertTriangle, Sparkles, Puzzle,
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

/// 插件 section ID 前缀。Sidebar/MainLayout 共享此约定。
/// 完整 section ID = `plugin:<plugin_id>`，例如 `plugin:com.example.certdeep`。
export const PLUGIN_SECTION_PREFIX = "plugin:";

/// 构造插件 section ID
export function pluginSectionId(pluginId: string): NavSection {
  return `${PLUGIN_SECTION_PREFIX}${pluginId}` as NavSection;
}

/// 从 section ID 提取 plugin_id（如果不是插件 section 返回 null）
export function pluginIdFromSection(section: string): string | null {
  if (section.startsWith(PLUGIN_SECTION_PREFIX)) {
    return section.slice(PLUGIN_SECTION_PREFIX.length);
  }
  return null;
}

export function Sidebar() {
  const { activeSection, setActiveSection, analysis } = useStore();

  // 从 analysis.plugins 派生动态 tab 列表。
  // - label: 优先用 manifest.ui_tab.label，fallback 到 plugin_name
  // - icon:  通过 resolvePluginIcon 解析 manifest.ui_tab.icon（未指定或不识别用默认）
  // - order: manifest.ui_tab.order（未指定视为 100，确保排在管理入口之前）
  const pluginTabs = (analysis?.plugins ?? [])
    .map((p: PluginResult) => ({
      id: pluginSectionId(p.plugin_id) as NavSection,
      label: p.ui_tab_label ?? p.plugin_name ?? p.plugin_id,
      Icon: resolvePluginIcon(p.ui_tab_icon),
      pluginId: p.plugin_id,
      hasError: p.error !== null && p.error !== undefined,
      order: p.ui_tab_order ?? 100,
    }))
    .sort((a, b) => a.order - b.order);

  const getBadge = (section: NavSection): number | null => {
    if (!analysis) return null;
    switch (section) {
      case "permissions": return analysis.permissions.summary.total || null;
      case "components": return analysis.components.stats.activities + analysis.components.stats.services + analysis.components.stats.receivers + analysis.components.stats.providers || null;
      case "native_libs": return analysis.native_libs.summary.total || null;
      case "dex": return analysis.dex.summary.total_dex_files || null;
      case "security": return analysis.security.issues.filter(i => i.severity === "critical" || i.severity === "high").length || null;
      case "plugins": return pluginTabs.length || null;
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

        {/* 分隔线 + 插件区 */}
        {pluginTabs.length > 0 && (
          <div className="pt-3 mt-3 border-t" style={{ borderColor: "var(--border-color)" }}>
            <div className="px-3 pb-1 text-[10px] font-semibold uppercase tracking-wider" style={{ color: "var(--text-tertiary)" }}>
              Plugins
            </div>
            {pluginTabs.map((tab) => {
              const Icon = tab.Icon;
              return (
                <div
                  key={tab.id}
                  className={`nav-item ${activeSection === tab.id ? "active" : ""}`}
                  onClick={() => setActiveSection(tab.id)}
                  title={tab.hasError ? "Plugin reported an error" : undefined}
                >
                  <Icon size={17} style={{ color: tab.hasError ? "var(--danger)" : "var(--accent)" }} />
                  <span className="flex-1 truncate">{tab.label}</span>
                  {tab.hasError && (
                    <span className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: "var(--danger)" }} />
                  )}
                </div>
              );
            })}
          </div>
        )}

        {/* 管理入口 */}
        <div className="pt-3 mt-3 border-t" style={{ borderColor: "var(--border-color)" }}>
          <div
            className={`nav-item ${activeSection === "plugins" ? "active" : ""}`}
            onClick={() => setActiveSection("plugins")}
          >
            <Puzzle size={17} />
            <span className="flex-1">Manage Plugins</span>
            {getBadge("plugins") !== null && (
              <span className="badge badge-neutral">{getBadge("plugins")}</span>
            )}
          </div>
        </div>
      </nav>
    </div>
  );
}
