import { useState } from "react";
import { useStore } from "../stores/useStore";
import { PageHeader, Section, DataTable } from "../components/common/DataTable";
import { protectionLevelClass, riskClass } from "../utils/format";
import { ShieldCheck, AlertTriangle, Shield, Lock, Info } from "lucide-react";

export function PermissionsPage() {
  const { analysis } = useStore();
  if (!analysis) return null;
  const p = analysis.permissions;
  const [filter, setFilter] = useState<string>("all");

  const filters = [
    { id: "all", label: "All", count: p.summary.total },
    { id: "dangerous", label: "Dangerous", count: p.summary.dangerous },
    { id: "special", label: "Special", count: p.summary.special },
    { id: "signature", label: "Signature", count: p.summary.signature },
    { id: "normal", label: "Normal", count: p.summary.normal },
    { id: "unknown", label: "Unknown", count: p.summary.unknown },
  ];

  const filtered = filter === "all" ? p.permissions : p.permissions.filter(perm => perm.protection_level === filter);

  // Highlight categories
  const highlightCategories = [
    { name: "Storage", icon: "💾", perms: ["READ_EXTERNAL_STORAGE", "WRITE_EXTERNAL_STORAGE", "MANAGE_EXTERNAL_STORAGE", "READ_MEDIA"] },
    { name: "Contacts", icon: "👤", perms: ["READ_CONTACTS", "WRITE_CONTACTS", "GET_ACCOUNTS"] },
    { name: "SMS", icon: "💬", perms: ["READ_SMS", "SEND_SMS", "RECEIVE_SMS", "RECEIVE_MMS"] },
    { name: "Camera", icon: "📷", perms: ["CAMERA"] },
    { name: "Microphone", icon: "🎤", perms: ["RECORD_AUDIO"] },
    { name: "Location", icon: "📍", perms: ["ACCESS_FINE_LOCATION", "ACCESS_COARSE_LOCATION", "ACCESS_BACKGROUND_LOCATION"] },
    { name: "Overlay", icon: "🪟", perms: ["SYSTEM_ALERT_WINDOW"] },
    { name: "Accessibility", icon: "♿", perms: ["BIND_ACCESSIBILITY_SERVICE"] },
    { name: "Notification", icon: "🔔", perms: ["POST_NOTIFICATIONS"] },
  ];

  const getPermCount = (keywords: string[]) => {
    return p.permissions.filter(perm => keywords.some(kw => perm.name.includes(kw))).length;
  };

  return (
    <div>
      <PageHeader title="Permission Analysis" subtitle={`${p.summary.total} permissions requested`} />

      {/* Summary cards */}
      <div className="grid grid-cols-2 md:grid-cols-5 gap-3 mb-6">
        <div className="stat-card">
          <ShieldCheck size={18} style={{ color: "var(--success)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Normal</span>
          <span className="text-lg font-bold" style={{ color: "var(--success)" }}>{p.summary.normal}</span>
        </div>
        <div className="stat-card">
          <AlertTriangle size={18} style={{ color: "var(--danger)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Dangerous</span>
          <span className="text-lg font-bold" style={{ color: "var(--danger)" }}>{p.summary.dangerous}</span>
        </div>
        <div className="stat-card">
          <Lock size={18} style={{ color: "var(--accent)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Signature</span>
          <span className="text-lg font-bold" style={{ color: "var(--accent)" }}>{p.summary.signature}</span>
        </div>
        <div className="stat-card">
          <Shield size={18} style={{ color: "var(--warning)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Special</span>
          <span className="text-lg font-bold" style={{ color: "var(--warning)" }}>{p.summary.special}</span>
        </div>
        <div className="stat-card">
          <Info size={18} style={{ color: "var(--text-tertiary)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Unknown</span>
          <span className="text-lg font-bold" style={{ color: "var(--text-tertiary)" }}>{p.summary.unknown}</span>
        </div>
      </div>

      {/* Highlight categories */}
      <Section title="Permission Categories">
        <div className="grid grid-cols-3 md:grid-cols-5 lg:grid-cols-9 gap-2">
          {highlightCategories.map((cat) => {
            const count = getPermCount(cat.perms);
            return (
              <div
                key={cat.name}
                className="card py-3 px-2 text-center transition-all"
                style={{
                  opacity: count > 0 ? 1 : 0.4,
                  borderColor: count > 0 ? "var(--accent)" : "var(--border-subtle)",
                }}
              >
                <div className="text-2xl mb-1">{cat.icon}</div>
                <div className="text-xs font-medium" style={{ color: "var(--text-primary)" }}>{cat.name}</div>
                <div className="text-xs" style={{ color: count > 0 ? "var(--accent)" : "var(--text-tertiary)" }}>{count > 0 ? `${count} requested` : "—"}</div>
              </div>
            );
          })}
        </div>
      </Section>

      {/* Permission table */}
      <Section title="All Permissions">
        <div className="flex gap-1 mb-4 p-1 rounded-lg inline-flex" style={{ backgroundColor: "var(--bg-tertiary)" }}>
          {filters.map((f) => (
            <button
              key={f.id}
              onClick={() => setFilter(f.id)}
              className={`px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${filter === f.id ? "text-white" : ""}`}
              style={filter === f.id ? { backgroundColor: "var(--accent)" } : { color: "var(--text-secondary)" }}
            >
              {f.label} ({f.count})
            </button>
          ))}
        </div>

        <DataTable
          data={filtered}
          searchKeys={["name", "category", "description"]}
          pageSize={50}
          expandable
          expandRender={(item) => (
            <div className="space-y-2">
              <div>
                <span className="text-xs font-semibold" style={{ color: "var(--text-secondary)" }}>Description: </span>
                <span className="text-xs" style={{ color: "var(--text-primary)" }}>{item.description}</span>
              </div>
              <div>
                <span className="text-xs font-semibold" style={{ color: "var(--text-secondary)" }}>Recommended Usage: </span>
                <span className="text-xs" style={{ color: "var(--text-primary)" }}>{item.recommended_usage}</span>
              </div>
            </div>
          )}
          columns={[
            {
              key: "name",
              label: "Permission",
              render: (item) => (
                <div className="flex flex-col">
                  <span className="font-mono text-xs" style={{ color: "var(--text-primary)" }}>{item.name}</span>
                  <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>{item.category}</span>
                </div>
              ),
            },
            {
              key: "protection_level",
              label: "Protection",
              render: (item) => <span className={`badge ${protectionLevelClass(item.protection_level)}`}>{item.protection_level}</span>,
            },
            {
              key: "risk_level",
              label: "Risk",
              render: (item) => <span className={`badge ${riskClass(item.risk_level)}`}>{item.risk_level}</span>,
            },
          ]}
        />
      </Section>
    </div>
  );
}
