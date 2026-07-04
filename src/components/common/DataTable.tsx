import { useState, useMemo } from "react";
import { Search, ChevronDown, ChevronRight } from "lucide-react";

interface DataTableProps<T> {
  data: T[];
  columns: {
    key: keyof T | string;
    label: string;
    render?: (item: T) => React.ReactNode;
    sortable?: boolean;
    width?: string;
  }[];
  searchable?: boolean;
  searchPlaceholder?: string;
  searchKeys?: (keyof T)[];
  expandable?: boolean;
  expandRender?: (item: T) => React.ReactNode;
  pageSize?: number;
}

export function DataTable<T extends Record<string, any>>({
  data,
  columns,
  searchable = true,
  searchPlaceholder = "Search...",
  searchKeys,
  expandable = false,
  expandRender,
  pageSize = 50,
}: DataTableProps<T>) {
  const [search, setSearch] = useState("");
  const [sortKey, setSortKey] = useState<string | null>(null);
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");
  const [expandedRows, setExpandedRows] = useState<Set<number>>(new Set());
  const [page, setPage] = useState(0);

  const filtered = useMemo(() => {
    let result = data;
    if (search && searchKeys) {
      const lower = search.toLowerCase();
      result = result.filter((item) =>
        searchKeys.some((key) =>
          String(item[key] ?? "").toLowerCase().includes(lower)
        )
      );
    } else if (search) {
      const lower = search.toLowerCase();
      result = result.filter((item) =>
        Object.values(item).some((v) => String(v ?? "").toLowerCase().includes(lower))
      );
    }

    if (sortKey) {
      result = [...result].sort((a, b) => {
        const av = a[sortKey];
        const bv = b[sortKey];
        if (typeof av === "number" && typeof bv === "number") {
          return sortDir === "asc" ? av - bv : bv - av;
        }
        const cmp = String(av ?? "").localeCompare(String(bv ?? ""));
        return sortDir === "asc" ? cmp : -cmp;
      });
    }

    return result;
  }, [data, search, searchKeys, sortKey, sortDir]);

  const paged = filtered.slice(page * pageSize, (page + 1) * pageSize);
  const totalPages = Math.ceil(filtered.length / pageSize);

  const toggleSort = (key: string) => {
    if (sortKey === key) {
      setSortDir(sortDir === "asc" ? "desc" : "asc");
    } else {
      setSortKey(key);
      setSortDir("asc");
    }
  };

  const toggleExpand = (index: number) => {
    const next = new Set(expandedRows);
    if (next.has(index)) {
      next.delete(index);
    } else {
      next.add(index);
    }
    setExpandedRows(next);
  };

  return (
    <div>
      {searchable && (
        <div className="mb-3 relative max-w-sm">
          <Search size={15} className="absolute left-3 top-1/2 -translate-y-1/2" style={{ color: "var(--text-tertiary)" }} />
          <input
            type="text"
            placeholder={searchPlaceholder}
            value={search}
            onChange={(e) => { setSearch(e.target.value); setPage(0); }}
            className="input pl-9"
          />
        </div>
      )}

      <div className="overflow-x-auto rounded-lg border" style={{ borderColor: "var(--border-subtle)" }}>
        <table className="w-full text-sm">
          <thead>
            <tr style={{ backgroundColor: "var(--bg-secondary)" }}>
              {expandable && <th className="w-10 px-3 py-2.5" />}
              {columns.map((col) => (
                <th
                  key={String(col.key)}
                  className="px-3 py-2.5 text-left font-semibold text-xs uppercase tracking-wide cursor-pointer"
                  style={{ color: "var(--text-secondary)", width: col.width }}
                  onClick={() => col.sortable !== false && toggleSort(String(col.key))}
                >
                  <div className="flex items-center gap-1">
                    {col.label}
                    {sortKey === String(col.key) && (
                      <span style={{ color: "var(--accent)" }}>{sortDir === "asc" ? "▲" : "▼"}</span>
                    )}
                  </div>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {paged.length === 0 ? (
              <tr>
                <td colSpan={columns.length + (expandable ? 1 : 0)} className="px-3 py-8 text-center" style={{ color: "var(--text-tertiary)" }}>
                  No results found
                </td>
              </tr>
            ) : (
              paged.map((item, i) => {
                const realIndex = page * pageSize + i;
                const isExpanded = expandedRows.has(realIndex);
                return (
                  <>
                    <tr key={realIndex} className="table-row">
                      {expandable && (
                        <td className="px-3 py-2.5 cursor-pointer" onClick={() => toggleExpand(realIndex)}>
                          {isExpanded ? <ChevronDown size={15} /> : <ChevronRight size={15} />}
                        </td>
                      )}
                      {columns.map((col) => (
                        <td key={String(col.key)} className="px-3 py-2.5" style={{ color: "var(--text-primary)" }}>
                          {col.render ? col.render(item) : String(item[col.key] ?? "")}
                        </td>
                      ))}
                    </tr>
                    {expandable && isExpanded && expandRender && (
                      <tr key={`${realIndex}-expand`}>
                        <td colSpan={columns.length + 1} className="px-6 py-3" style={{ backgroundColor: "var(--bg-secondary)" }}>
                          {expandRender(item)}
                        </td>
                      </tr>
                    )}
                  </>
                );
              })
            )}
          </tbody>
        </table>
      </div>

      {totalPages > 1 && (
        <div className="flex items-center justify-between mt-3">
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>
            {filtered.length} items · Page {page + 1} of {totalPages}
          </span>
          <div className="flex gap-1">
            <button
              onClick={() => setPage(Math.max(0, page - 1))}
              disabled={page === 0}
              className="btn btn-secondary text-xs px-3 py-1.5 disabled:opacity-40"
            >
              Previous
            </button>
            <button
              onClick={() => setPage(Math.min(totalPages - 1, page + 1))}
              disabled={page >= totalPages - 1}
              className="btn btn-secondary text-xs px-3 py-1.5 disabled:opacity-40"
            >
              Next
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export function PageHeader({ title, subtitle, children }: { title: string; subtitle?: string; children?: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between mb-6">
      <div>
        <h1 className="text-xl font-bold" style={{ color: "var(--text-primary)" }}>{title}</h1>
        {subtitle && <p className="text-sm mt-0.5" style={{ color: "var(--text-secondary)" }}>{subtitle}</p>}
      </div>
      {children}
    </div>
  );
}

export function InfoGrid({ items }: { items: { label: string; value: React.ReactNode; badge?: string }[] }) {
  return (
    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
      {items.map((item, i) => (
        <div key={i} className="stat-card">
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>{item.label}</span>
          <span className="text-sm font-semibold" style={{ color: "var(--text-primary)" }}>
            {item.value}
            {item.badge && <span className={`badge ${item.badge} ml-2`}>{}</span>}
          </span>
        </div>
      ))}
    </div>
  );
}

export function Section({ title, children, action }: { title: string; children: React.ReactNode; action?: React.ReactNode }) {
  return (
    <div className="mb-6">
      <div className="flex items-center justify-between mb-3">
        <h2 className="text-sm font-semibold uppercase tracking-wide" style={{ color: "var(--text-secondary)" }}>{title}</h2>
        {action}
      </div>
      {children}
    </div>
  );
}
