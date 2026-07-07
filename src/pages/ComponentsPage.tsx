import { useState } from "react";
import { useStore } from "../stores/useStore";
import { PageHeader, Section, DataTable } from "../components/common/DataTable";
import { Boxes, Activity, Server, Radio, Database, AlertCircle } from "lucide-react";
import type { Component } from "../types";

export function ComponentsPage() {
  const { analysis } = useStore();
  if (!analysis) return null;
  const c = analysis.components;
  const [activeTab, setActiveTab] = useState<"activities" | "services" | "receivers" | "providers" | "exported">("activities");

  const stats = [
    { label: "Activities", count: c.stats.activities, icon: <Activity size={18} />, color: "var(--accent)" },
    { label: "Services", count: c.stats.services, icon: <Server size={18} />, color: "var(--success)" },
    { label: "Receivers", count: c.stats.receivers, icon: <Radio size={18} />, color: "var(--warning)" },
    { label: "Providers", count: c.stats.providers, icon: <Database size={18} />, color: "var(--accent)" },
  ];

  const tabs = [
    { id: "activities" as const, label: "Activities", count: c.stats.activities },
    { id: "services" as const, label: "Services", count: c.stats.services },
    { id: "receivers" as const, label: "Receivers", count: c.stats.receivers },
    { id: "providers" as const, label: "Providers", count: c.stats.providers },
    { id: "exported" as const, label: "Exported", count: c.exported_components.length },
  ];

  const componentData: Record<string, Component[]> = {
    activities: c.activities,
    services: c.services,
    receivers: c.receivers,
    providers: c.providers,
  };

  return (
    <div>
      <PageHeader title="Component Analysis" subtitle={`${c.stats.activities + c.stats.services + c.stats.receivers + c.stats.providers} total components`} />

      {/* Stats */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-6">
        {stats.map((s) => (
          <div key={s.label} className="card flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg flex items-center justify-center" style={{ backgroundColor: `${s.color}15` }}>
              <div style={{ color: s.color }}>{s.icon}</div>
            </div>
            <div>
              <div className="text-2xl font-bold" style={{ color: "var(--text-primary)" }}>{s.count}</div>
              <div className="text-xs" style={{ color: "var(--text-tertiary)" }}>{s.label}</div>
            </div>
          </div>
        ))}
      </div>

      {/* Exported warning */}
      {c.stats.exported > 0 && (
        <div className="card mb-6 flex items-center gap-3" style={{ borderColor: "var(--warning)" }}>
          <AlertCircle size={20} style={{ color: "var(--warning)" }} />
          <div>
            <p className="text-sm font-medium" style={{ color: "var(--text-primary)" }}>{c.stats.exported} exported components detected</p>
            <p className="text-xs" style={{ color: "var(--text-secondary)" }}>Exported components can be accessed by other apps. Review for security implications.</p>
          </div>
        </div>
      )}

      <Section title="Components">
        <div className="flex gap-1 mb-4 p-1 rounded-lg inline-flex" style={{ backgroundColor: "var(--bg-tertiary)" }}>
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${activeTab === tab.id ? "text-white" : ""}`}
              style={activeTab === tab.id ? { backgroundColor: "var(--accent)" } : { color: "var(--text-secondary)" }}
            >
              {tab.label} ({tab.count})
            </button>
          ))}
        </div>

        {activeTab === "exported" ? (
          <DataTable
            data={c.exported_components}
            getRowId={(item) => `${item.component_type}:${item.name}`}
            searchKeys={["name", "component_type"]}
            columns={[
              {
                key: "name",
                label: "Component",
                render: (item) => (
                  <span className="font-mono text-xs" style={{ color: "var(--text-primary)" }}>
                    {item.name.split(".").pop() || item.name}
                    <span className="ml-1" style={{ color: "var(--text-tertiary)" }}>({item.name})</span>
                  </span>
                ),
              },
              {
                key: "component_type",
                label: "Type",
                render: (item) => <span className="badge badge-info">{item.component_type}</span>,
              },
              {
                key: "permission",
                label: "Permission",
                render: (item) => item.permission
                  ? <span className="text-xs font-mono" style={{ color: "var(--warning)" }}>{item.permission.split(".").pop()}</span>
                  : <span className="badge badge-danger">None</span>,
              },
              {
                key: "has_intent_filter",
                label: "Intent Filter",
                render: (item) => item.has_intent_filter
                  ? <span className="badge badge-success">Yes</span>
                  : <span style={{ color: "var(--text-tertiary)" }}>No</span>,
              },
            ]}
          />
        ) : (
          <DataTable
            data={componentData[activeTab]}
            getRowId={(item) => item.name}
            searchKeys={["name", "permission"]}
            expandable
            expandRender={(item: Component) => (
              <div className="space-y-3">
                {item.intent_filters.length > 0 && (
                  <div>
                    <p className="text-xs font-semibold mb-1" style={{ color: "var(--text-secondary)" }}>Intent Filters ({item.intent_filters.length})</p>
                    {item.intent_filters.map((f, i) => (
                      <div key={i} className="text-xs space-y-1 p-2 rounded" style={{ backgroundColor: "var(--bg-tertiary)" }}>
                        {f.actions.length > 0 && <div><span style={{ color: "var(--text-tertiary)" }}>Actions:</span> {f.actions.map(a => <span key={a} className="badge badge-info mr-1">{a.split(".").pop()}</span>)}</div>}
                        {f.categories.length > 0 && <div><span style={{ color: "var(--text-tertiary)" }}>Categories:</span> {f.categories.map(cat => <span key={cat} className="badge badge-neutral mr-1">{cat.split(".").pop()}</span>)}</div>}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
            columns={[
              {
                key: "name",
                label: "Name",
                render: (item) => (
                  <span className="font-mono text-xs" style={{ color: "var(--text-primary)" }}>
                    {item.name.split(".").pop() || item.name}
                  </span>
                ),
              },
              {
                key: "exported",
                label: "Exported",
                render: (item) => item.exported
                  ? <span className="badge badge-danger">Yes</span>
                  : <span className="badge badge-neutral">No</span>,
              },
              {
                key: "permission",
                label: "Permission",
                render: (item) => item.permission
                  ? <span className="text-xs font-mono" style={{ color: "var(--warning)" }}>{item.permission.split(".").pop()}</span>
                  : <span style={{ color: "var(--text-tertiary)" }}>—</span>,
              },
              {
                key: "intent_filters",
                label: "Filters",
                render: (item) => item.intent_filters.length > 0
                  ? <span className="badge badge-info">{item.intent_filters.length}</span>
                  : <span style={{ color: "var(--text-tertiary)" }}>—</span>,
              },
            ]}
          />
        )}
      </Section>
    </div>
  );
}
