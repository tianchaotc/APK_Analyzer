import { useState } from "react";
import { useStore } from "../stores/useStore";
import { PageHeader, Section, DataTable } from "../components/common/DataTable";
import { copyToClipboard } from "../utils/format";
import { Copy, Code2, Boxes } from "lucide-react";
import type { Component } from "../types";

export function ManifestPage() {
  const { analysis } = useStore();
  if (!analysis) return null;
  const m = analysis.manifest;
  const [view, setView] = useState<"components" | "raw">("components");
  const [activeTab, setActiveTab] = useState<"activities" | "services" | "receivers" | "providers">("activities");

  const componentData: Record<string, Component[]> = {
    activities: m.activities,
    services: m.services,
    receivers: m.receivers,
    providers: m.providers,
  };

  const tabs = [
    { id: "activities" as const, label: "Activities", count: m.activities.length },
    { id: "services" as const, label: "Services", count: m.services.length },
    { id: "receivers" as const, label: "Receivers", count: m.receivers.length },
    { id: "providers" as const, label: "Providers", count: m.providers.length },
  ];

  return (
    <div>
      <PageHeader title="Manifest Analysis" subtitle="Parsed AndroidManifest.xml content">
        <div className="flex gap-1 p-1 rounded-lg" style={{ backgroundColor: "var(--bg-tertiary)" }}>
          <button
            onClick={() => setView("components")}
            className={`px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${view === "components" ? "text-white" : ""}`}
            style={view === "components" ? { backgroundColor: "var(--accent)" } : { color: "var(--text-secondary)" }}
          >
            Components
          </button>
          <button
            onClick={() => setView("raw")}
            className={`px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${view === "raw" ? "text-white" : ""}`}
            style={view === "raw" ? { backgroundColor: "var(--accent)" } : { color: "var(--text-secondary)" }}
          >
            Raw XML
          </button>
        </div>
      </PageHeader>

      {view === "raw" ? (
        <div className="card overflow-hidden">
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2">
              <Code2 size={16} style={{ color: "var(--accent)" }} />
              <span className="text-sm font-medium" style={{ color: "var(--text-secondary)" }}>AndroidManifest.xml</span>
            </div>
            <button onClick={() => copyToClipboard(m.raw_xml)} className="btn btn-secondary text-xs">
              <Copy size={13} /> Copy
            </button>
          </div>
          <pre className="text-xs overflow-auto max-h-[600px] p-4 rounded-lg" style={{ backgroundColor: "var(--bg-secondary)", color: "var(--text-primary)", fontFamily: "monospace" }}>
            {m.raw_xml}
          </pre>
        </div>
      ) : (
        <>
          <Section title="Manifest Properties">
            <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
              {[
                { label: "Package", value: m.package },
                { label: "Version Code", value: m.version_code },
                { label: "Version Name", value: m.version_name },
                { label: "Min SDK", value: String(m.min_sdk) },
                { label: "Target SDK", value: String(m.target_sdk) },
                { label: "Compile SDK", value: String(m.compile_sdk) },
                { label: "Launch Activity", value: m.launch_activity ? m.launch_activity.split(".").pop() || m.launch_activity : "—" },
                { label: "Queries", value: String(m.queries.length) },
              ].map((s) => (
                <div key={s.label} className="stat-card">
                  <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>{s.label}</span>
                  <span className="text-sm font-semibold truncate" style={{ color: "var(--text-primary)" }}>{s.value || "—"}</span>
                </div>
              ))}
            </div>
          </Section>

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

            <DataTable
              data={componentData[activeTab]}
              searchPlaceholder={`Search ${activeTab}...`}
              searchKeys={["name", "permission"]}
              expandable
              expandRender={(item: Component) => (
                <div className="space-y-3">
                  {item.intent_filters.length > 0 && (
                    <div>
                      <p className="text-xs font-semibold mb-1" style={{ color: "var(--text-secondary)" }}>Intent Filters ({item.intent_filters.length})</p>
                      {item.intent_filters.map((f, i) => (
                        <div key={i} className="text-xs space-y-1 p-2 rounded" style={{ backgroundColor: "var(--bg-tertiary)" }}>
                          {f.actions.length > 0 && <div><span style={{ color: "var(--text-tertiary)" }}>Actions:</span> {f.actions.join(", ")}</div>}
                          {f.categories.length > 0 && <div><span style={{ color: "var(--text-tertiary)" }}>Categories:</span> {f.categories.join(", ")}</div>}
                          {f.data_schemes.length > 0 && <div><span style={{ color: "var(--text-tertiary)" }}>Schemes:</span> {f.data_schemes.join(", ")}</div>}
                          {f.data_mime_types.length > 0 && <div><span style={{ color: "var(--text-tertiary)" }}>MimeTypes:</span> {f.data_mime_types.join(", ")}</div>}
                        </div>
                      ))}
                    </div>
                  )}
                  {item.meta_data.length > 0 && (
                    <div>
                      <p className="text-xs font-semibold mb-1" style={{ color: "var(--text-secondary)" }}>Meta-data</p>
                      {item.meta_data.map((md, i) => (
                        <div key={i} className="text-xs" style={{ color: "var(--text-secondary)" }}>
                          {md.name}: {md.value || md.resource || "—"}
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
                      <span className="ml-1" style={{ color: "var(--text-tertiary)" }}>
                        ({item.name})
                      </span>
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
                  label: "Intent Filters",
                  render: (item) => item.intent_filters.length > 0
                    ? <span className="badge badge-info">{item.intent_filters.length}</span>
                    : <span style={{ color: "var(--text-tertiary)" }}>—</span>,
                },
              ]}
            />
          </Section>

          {m.uses_features.length > 0 && (
            <Section title="Uses Features">
              <div className="flex flex-wrap gap-2">
                {m.uses_features.map((f, i) => (
                  <div key={i} className="card py-2 px-3 flex items-center gap-2">
                    <span className="text-xs font-mono" style={{ color: "var(--text-primary)" }}>{f.name}</span>
                    {f.required && <span className="badge badge-danger">Required</span>}
                  </div>
                ))}
              </div>
            </Section>
          )}

          {m.meta_data.length > 0 && (
            <Section title="Application Meta-data">
              <DataTable
                data={m.meta_data}
                searchKeys={["name", "value"]}
                pageSize={20}
                columns={[
                  { key: "name", label: "Name", render: (i) => <span className="font-mono text-xs">{i.name}</span> },
                  { key: "value", label: "Value", render: (i) => i.value || <span style={{ color: "var(--text-tertiary)" }}>—</span> },
                  { key: "resource", label: "Resource", render: (i) => i.resource || <span style={{ color: "var(--text-tertiary)" }}>—</span> },
                ]}
              />
            </Section>
          )}
        </>
      )}
    </div>
  );
}
