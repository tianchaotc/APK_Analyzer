import { useStore } from "../stores/useStore";
import { PageHeader, Section, DataTable } from "../components/common/DataTable";
import { formatFileSize, formatNumber } from "../utils/format";
import { Binary, Boxes, FunctionSquare, Database } from "lucide-react";

export function DexPage() {
  const { analysis } = useStore();
  if (!analysis) return null;
  const d = analysis.dex;

  return (
    <div>
      <PageHeader title="DEX Analysis" subtitle={`${d.summary.total_dex_files} DEX file(s) · ${formatNumber(d.summary.total_classes)} classes · ${formatNumber(d.summary.total_methods)} methods`} />

      {/* Summary */}
      <div className="grid grid-cols-2 md:grid-cols-5 gap-3 mb-6">
        <div className="stat-card">
          <Binary size={18} style={{ color: "var(--accent)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>DEX Files</span>
          <span className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{formatNumber(d.summary.total_dex_files)}</span>
        </div>
        <div className="stat-card">
          <Boxes size={18} style={{ color: "var(--accent)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Classes</span>
          <span className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{formatNumber(d.summary.total_classes)}</span>
        </div>
        <div className="stat-card">
          <FunctionSquare size={18} style={{ color: "var(--accent)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Methods</span>
          <span className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{formatNumber(d.summary.total_methods)}</span>
        </div>
        <div className="stat-card">
          <Database size={18} style={{ color: "var(--accent)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Fields</span>
          <span className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{formatNumber(d.summary.total_fields)}</span>
        </div>
        <div className="stat-card">
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Total Size</span>
          <span className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{formatFileSize(d.summary.total_size)}</span>
        </div>
      </div>

      {/* DEX files */}
      <Section title="DEX Files">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {d.dex_files.map((file) => (
            <div key={file.name} className="card">
              <div className="flex items-center justify-between mb-3">
                <span className="text-sm font-mono font-semibold" style={{ color: "var(--text-primary)" }}>{file.name}</span>
                <span className="text-sm font-bold" style={{ color: "var(--accent)" }}>{formatFileSize(file.size)}</span>
              </div>
              <div className="grid grid-cols-3 gap-2">
                <div className="text-center p-2 rounded" style={{ backgroundColor: "var(--bg-secondary)" }}>
                  <div className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{formatNumber(file.class_count)}</div>
                  <div className="text-xs" style={{ color: "var(--text-tertiary)" }}>Classes</div>
                </div>
                <div className="text-center p-2 rounded" style={{ backgroundColor: "var(--bg-secondary)" }}>
                  <div className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{formatNumber(file.method_count)}</div>
                  <div className="text-xs" style={{ color: "var(--text-tertiary)" }}>Methods</div>
                </div>
                <div className="text-center p-2 rounded" style={{ backgroundColor: "var(--bg-secondary)" }}>
                  <div className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{formatNumber(file.field_count)}</div>
                  <div className="text-xs" style={{ color: "var(--text-tertiary)" }}>Fields</div>
                </div>
              </div>
            </div>
          ))}
        </div>
      </Section>

      {/* Largest packages */}
      {d.largest_packages.length > 0 && (
        <Section title="Largest Packages">
          <DataTable
            data={d.largest_packages}
            searchKeys={["name"]}
            pageSize={20}
            columns={[
              { key: "name", label: "Package", render: (i) => <span className="font-mono text-xs">{i.name}</span> },
              { key: "class_count", label: "Classes", render: (i) => <span className="font-mono text-xs font-semibold">{formatNumber(i.class_count)}</span> },
              { key: "method_count", label: "Methods", render: (i) => <span className="font-mono text-xs">{formatNumber(i.method_count)}</span> },
              { key: "field_count", label: "Fields", render: (i) => <span className="font-mono text-xs">{formatNumber(i.field_count)}</span> },
            ]}
          />
        </Section>
      )}

      {/* All packages */}
      <Section title="All Packages">
        <DataTable
          data={d.packages}
          searchKeys={["name"]}
          pageSize={50}
          columns={[
            { key: "name", label: "Package", render: (i) => <span className="font-mono text-xs">{i.name}</span> },
            { key: "class_count", label: "Classes", render: (i) => <span className="font-mono text-xs font-semibold">{formatNumber(i.class_count)}</span> },
            { key: "method_count", label: "Methods", render: (i) => <span className="font-mono text-xs">{formatNumber(i.method_count)}</span> },
            { key: "field_count", label: "Fields", render: (i) => <span className="font-mono text-xs">{formatNumber(i.field_count)}</span> },
          ]}
        />
      </Section>
    </div>
  );
}
