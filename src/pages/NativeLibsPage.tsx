import { useStore } from "../stores/useStore";
import { PageHeader, Section, DataTable } from "../components/common/DataTable";
import { formatFileSize, formatNumber } from "../utils/format";
import { Cpu, HardDrive, Layers } from "lucide-react";

export function NativeLibsPage() {
  const { analysis } = useStore();
  if (!analysis) return null;
  const n = analysis.native_libs;

  return (
    <div>
      <PageHeader title="Native Library Analysis" subtitle={`${n.summary.total} libraries across ${n.summary.abis.length} ABIs · ${formatFileSize(n.summary.total_size)}`} />

      {/* Summary */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-6">
        <div className="stat-card">
          <Cpu size={18} style={{ color: "var(--accent)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Total Libraries</span>
          <span className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{formatNumber(n.summary.total)}</span>
        </div>
        <div className="stat-card">
          <HardDrive size={18} style={{ color: "var(--accent)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Total Size</span>
          <span className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{formatFileSize(n.summary.total_size)}</span>
        </div>
        <div className="stat-card">
          <Layers size={18} style={{ color: "var(--accent)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>ABIs</span>
          <span className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{n.summary.abis.length}</span>
        </div>
        <div className="stat-card">
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Avg per ABI</span>
          <span className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{n.summary.abis.length > 0 ? formatNumber(Math.round(n.summary.total / n.summary.abis.length)) : "0"}</span>
        </div>
      </div>

      {/* ABI groups */}
      <Section title="Libraries by ABI">
        <div className="space-y-4">
          {n.by_abi.map((group) => (
            <div key={group.abi} className="card">
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-3">
                  <span className="badge badge-info">{group.abi}</span>
                  <span className="text-sm" style={{ color: "var(--text-secondary)" }}>{group.count} libraries</span>
                </div>
                <span className="text-sm font-bold" style={{ color: "var(--accent)" }}>{formatFileSize(group.total_size)}</span>
              </div>
              <DataTable
                data={group.libraries}
                searchKeys={["file_name"]}
                pageSize={20}
                columns={[
                  { key: "file_name", label: "Library", render: (i) => <span className="font-mono text-xs">{i.file_name}</span> },
                  { key: "architecture", label: "Architecture", render: (i) => <span className="text-xs" style={{ color: "var(--text-secondary)" }}>{i.architecture}</span> },
                  { key: "size", label: "Size", render: (i) => <span className="font-mono text-xs font-semibold">{formatFileSize(i.size)}</span> },
                  { key: "compressed_size", label: "Compressed", render: (i) => <span className="font-mono text-xs">{formatFileSize(i.compressed_size)}</span> },
                  { key: "compression", label: "Ratio", render: (i) => <span className="badge badge-neutral">{i.compression}</span> },
                ]}
              />
            </div>
          ))}
        </div>
      </Section>

      {/* All libraries table */}
      <Section title="All Libraries">
        <DataTable
          data={n.libraries}
          searchKeys={["file_name", "abi", "architecture"]}
          pageSize={50}
          columns={[
            { key: "file_name", label: "Library", render: (i) => <span className="font-mono text-xs">{i.file_name}</span> },
            { key: "abi", label: "ABI", render: (i) => <span className="badge badge-info">{i.abi}</span> },
            { key: "architecture", label: "Architecture", render: (i) => <span className="text-xs" style={{ color: "var(--text-secondary)" }}>{i.architecture}</span> },
            { key: "size", label: "Size", render: (i) => <span className="font-mono text-xs font-semibold">{formatFileSize(i.size)}</span> },
            { key: "compression", label: "Compression", render: (i) => <span className="badge badge-neutral">{i.compression}</span> },
          ]}
        />
      </Section>
    </div>
  );
}
