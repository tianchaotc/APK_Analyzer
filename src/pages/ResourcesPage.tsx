import { useStore } from "../stores/useStore";
import { PageHeader, Section, DataTable } from "../components/common/DataTable";
import { formatFileSize, formatNumber } from "../utils/format";
import { Image, Copy, Layers, AlertTriangle, HardDrive } from "lucide-react";

export function ResourcesPage() {
  const { analysis } = useStore();
  if (!analysis) return null;
  const r = analysis.resources;

  return (
    <div>
      <PageHeader title="Resource Analysis" subtitle={`${formatNumber(r.summary.total)} resources · ${formatFileSize(r.summary.total_size)}`} />

      {/* Summary */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-6">
        <div className="stat-card">
          <HardDrive size={18} style={{ color: "var(--accent)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Total Resources</span>
          <span className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{formatNumber(r.summary.total)}</span>
        </div>
        <div className="stat-card">
          <Layers size={18} style={{ color: "var(--accent)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Total Size</span>
          <span className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{formatFileSize(r.summary.total_size)}</span>
        </div>
        <div className="stat-card">
          <Image size={18} style={{ color: "var(--accent)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Images</span>
          <span className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{formatNumber(r.image_stats.total_images)}</span>
        </div>
        <div className="stat-card">
          <AlertTriangle size={18} style={{ color: r.duplicate_resources.length > 0 ? "var(--warning)" : "var(--success)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Duplicates</span>
          <span className="text-lg font-bold" style={{ color: r.duplicate_resources.length > 0 ? "var(--warning)" : "var(--success)" }}>{r.duplicate_resources.length}</span>
        </div>
      </div>

      {/* By type */}
      <Section title="Resources by Type">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {r.by_type.map((group) => (
            <div key={group.type_name} className="card">
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-semibold capitalize" style={{ color: "var(--text-primary)" }}>{group.type_name}</span>
                  <span className="badge badge-neutral">{group.count}</span>
                </div>
                <span className="text-sm font-bold" style={{ color: "var(--accent)" }}>{formatFileSize(group.total_size)}</span>
              </div>
              {/* Bar chart */}
              <div className="h-2 rounded-full overflow-hidden mb-3" style={{ backgroundColor: "var(--bg-tertiary)" }}>
                <div className="h-full rounded-full" style={{ width: `${(group.total_size / r.summary.total_size * 100)}%`, backgroundColor: "var(--accent)" }} />
              </div>
              <div className="text-xs space-y-1">
                {group.entries.slice(0, 5).map((entry, i) => (
                  <div key={i} className="flex justify-between">
                    <span className="truncate" style={{ color: "var(--text-secondary)" }}>{entry.name}</span>
                    <span style={{ color: "var(--text-tertiary)" }}>{formatFileSize(entry.size)}</span>
                  </div>
                ))}
                {group.entries.length > 5 && (
                  <div className="text-xs" style={{ color: "var(--text-tertiary)" }}>+ {group.entries.length - 5} more</div>
                )}
              </div>
            </div>
          ))}
        </div>
      </Section>

      {/* Image stats */}
      {r.image_stats.total_images > 0 && (
        <Section title="Image Statistics">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-3">
            {r.image_stats.by_format.map((fmt) => (
              <div key={fmt.format} className="stat-card">
                <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>{fmt.format}</span>
                <span className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{fmt.count}</span>
                <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>{formatFileSize(fmt.total_size)}</span>
              </div>
            ))}
          </div>
          <DataTable
            data={r.image_stats.largest_images}
            searchKeys={["name", "path"]}
            pageSize={20}
            columns={[
              { key: "name", label: "Image", render: (i) => <span className="font-mono text-xs">{i.name}</span> },
              { key: "path", label: "Path", render: (i) => <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>{i.path}</span> },
              { key: "size", label: "Size", render: (i) => <span className="font-mono text-xs">{formatFileSize(i.size)}</span> },
            ]}
          />
        </Section>
      )}

      {/* Largest resources */}
      <Section title="Largest Resources">
        <DataTable
          data={r.largest_resources}
          searchKeys={["name", "path"]}
          pageSize={30}
          columns={[
            { key: "name", label: "Name", render: (i) => <span className="font-mono text-xs">{i.name}</span> },
            { key: "resource_type", label: "Type", render: (i) => <span className="badge badge-neutral capitalize">{i.resource_type}</span> },
            { key: "path", label: "Path", render: (i) => <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>{i.path}</span> },
            { key: "size", label: "Size", render: (i) => <span className="font-mono text-xs font-semibold">{formatFileSize(i.size)}</span> },
            { key: "compression", label: "Compression", render: (i) => i.compression ? <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>{i.compression}</span> : <span style={{ color: "var(--text-tertiary)" }}>—</span> },
          ]}
        />
      </Section>

      {/* Duplicates */}
      {r.duplicate_resources.length > 0 && (
        <Section title="Duplicate Resources">
          <div className="space-y-2">
            {r.duplicate_resources.slice(0, 20).map((dup, i) => (
              <div key={i} className="card">
                <div className="flex items-center justify-between mb-2">
                  <span className="font-mono text-sm font-medium" style={{ color: "var(--text-primary)" }}>{dup.name}</span>
                  <span className="text-xs" style={{ color: "var(--warning)" }}>{dup.paths.length} copies · {formatFileSize(dup.total_size)}</span>
                </div>
                <div className="text-xs space-y-1">
                  {dup.paths.slice(0, 5).map((path, j) => (
                    <div key={j} style={{ color: "var(--text-tertiary)" }}>{path}</div>
                  ))}
                  {dup.paths.length > 5 && <div style={{ color: "var(--text-tertiary)" }}>+ {dup.paths.length - 5} more</div>}
                </div>
              </div>
            ))}
          </div>
        </Section>
      )}
    </div>
  );
}
