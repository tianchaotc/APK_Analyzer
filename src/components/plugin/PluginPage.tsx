import type { PluginResult, PluginUiSchema, UiSection } from "../../types";
import { AlertTriangle, Clock, CheckCircle2, XCircle } from "lucide-react";

type JsonRecord = Record<string, unknown>;

interface PluginPageProps {
  result: PluginResult;
}

/// 声明式 schema 驱动的插件结果渲染器。
/// 根据 ui_schema 的 sections 类型渲染表格/卡片/统计网格/Markdown/柱状图。
export function PluginPage({ result }: PluginPageProps) {
  if (result.error) {
    return (
      <div className="space-y-4">
        <PluginHeader result={result} />
        <div className="rounded-lg p-4 border" style={{ borderColor: "var(--danger)", backgroundColor: "var(--danger-bg)" }}>
          <div className="flex items-center gap-2 mb-2">
            <XCircle size={16} color="var(--danger)" />
            <span className="font-semibold" style={{ color: "var(--danger)" }}>Analysis Failed</span>
          </div>
          <p className="text-sm" style={{ color: "var(--text-secondary)" }}>{result.error}</p>
        </div>
      </div>
    );
  }

  const schema = parsePluginUiSchema(result.ui_schema);

  if (!schema) return <RawPluginData result={result} message="Plugin returned no valid UI schema. Raw data:" />;

  return (
    <div className="space-y-4">
      <PluginHeader result={result} />
      <div className="space-y-4">
        {schema.sections.map((section, i) => (
          <SectionRenderer key={i} section={section} data={result.data} />
        ))}
      </div>
    </div>
  );
}

function RawPluginData({ result, message }: { result: PluginResult; message: string }) {
  return (
    <div className="space-y-4">
      <PluginHeader result={result} />
      <div className="rounded-lg p-4 border" style={{ borderColor: "var(--border-color)" }}>
        <p className="text-sm mb-3" style={{ color: "var(--text-secondary)" }}>
          {message}
        </p>
        <pre className="text-xs overflow-auto p-3 rounded" style={{ backgroundColor: "var(--bg-tertiary)", color: "var(--text-primary)" }}>
          {JSON.stringify(result.data, null, 2)}
        </pre>
      </div>
    </div>
  );
}

function PluginHeader({ result }: { result: PluginResult }) {
  return (
    <div className="flex items-center justify-between">
      <div>
        <h1 className="text-xl font-bold" style={{ color: "var(--text-primary)" }}>{result.plugin_name}</h1>
        <p className="text-xs" style={{ color: "var(--text-tertiary)" }}>{result.plugin_id}</p>
      </div>
      <div className="flex items-center gap-3 text-xs" style={{ color: "var(--text-tertiary)" }}>
        {result.error ? (
          <span className="flex items-center gap-1"><XCircle size={13} /> Error</span>
        ) : (
          <span className="flex items-center gap-1"><CheckCircle2 size={13} color="var(--success)" /> Success</span>
        )}
        <span className="flex items-center gap-1"><Clock size={13} /> {result.duration_ms} ms</span>
      </div>
    </div>
  );
}

function SectionRenderer({ section, data }: { section: UiSection; data: unknown }) {
  switch (section.type) {
    case "table":
      return <TableSection section={section} data={data} />;
    case "stat_grid":
      return <StatGridSection section={section} data={data} />;
    case "markdown":
      return <MarkdownSection section={section} data={data} />;
    case "cards":
      return <CardsSection section={section} data={data} />;
    case "chart_bar":
      return <ChartBarSection section={section} data={data} />;
    default:
      return null;
  }
}

type TableSectionT = Extract<UiSection, { type: "table" }>;
function TableSection({ section, data }: { section: TableSectionT; data: unknown }) {
  const rows = getRecordArray(data, section.data_key);
  return (
    <div className="rounded-lg border" style={{ borderColor: "var(--border-color)" }}>
      <div className="overflow-auto">
        <table className="w-full text-sm">
          <thead>
            <tr style={{ backgroundColor: "var(--bg-secondary)" }}>
              {section.columns.map((col) => (
                <th
                  key={col.key}
                  className="text-left px-3 py-2 font-semibold"
                  style={{ color: "var(--text-secondary)", width: col.width }}
                >
                  {col.label}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.length === 0 ? (
              <tr>
                <td colSpan={section.columns.length} className="px-3 py-4 text-center text-xs" style={{ color: "var(--text-tertiary)" }}>
                  No data
                </td>
              </tr>
            ) : (
              rows.map((row, i) => (
                <tr key={i} className="border-t" style={{ borderColor: "var(--border-color)" }}>
                  {section.columns.map((col) => (
                    <td key={col.key} className="px-3 py-2" style={{ color: "var(--text-primary)" }}>
                      {formatValue(row[col.key])}
                    </td>
                  ))}
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}

type StatGridSectionT = Extract<UiSection, { type: "stat_grid" }>;
function StatGridSection({ section, data }: { section: StatGridSectionT; data: unknown }) {
  const source = getRecord(getRecordValue(data, section.data_key)) ?? getRecord(data);
  return (
    <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
      {section.metrics.map((metric) => {
        const value = source?.[metric.key];
        return (
          <div key={metric.key} className="rounded-lg p-3 border" style={{ borderColor: "var(--border-color)", backgroundColor: "var(--bg-secondary)" }}>
            <div className="text-xs uppercase tracking-wide" style={{ color: "var(--text-tertiary)" }}>{metric.label}</div>
            <div className="text-lg font-semibold mt-1" style={{ color: "var(--text-primary)" }}>
              {formatValue(value)}{metric.unit ? <span className="text-xs ml-1" style={{ color: "var(--text-tertiary)" }}>{metric.unit}</span> : null}
            </div>
          </div>
        );
      })}
    </div>
  );
}

type MarkdownSectionT = Extract<UiSection, { type: "markdown" }>;
function MarkdownSection({ section, data }: { section: MarkdownSectionT; data: unknown }) {
  const value = getRecordValue(data, section.data_key);
  const text = typeof value === "string" ? value : "";
  return (
    <div className="rounded-lg p-4 border prose prose-sm max-w-none" style={{ borderColor: "var(--border-color)" }}>
      <pre className="whitespace-pre-wrap text-sm" style={{ color: "var(--text-primary)", fontFamily: "inherit" }}>
        {text}
      </pre>
    </div>
  );
}

type CardsSectionT = Extract<UiSection, { type: "cards" }>;
function CardsSection({ section, data }: { section: CardsSectionT; data: unknown }) {
  const items = getRecordArray(data, section.data_key);
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
      {items.map((item, i) => (
        <div key={i} className="rounded-lg p-3 border" style={{ borderColor: "var(--border-color)" }}>
          <div className="font-semibold text-sm" style={{ color: "var(--text-primary)" }}>
            {formatValue(item[section.card.title_key])}
          </div>
          <div className="text-xs mt-1" style={{ color: "var(--text-secondary)" }}>
            {formatValue(item[section.card.body_key])}
          </div>
        </div>
      ))}
    </div>
  );
}

type ChartBarSectionT = Extract<UiSection, { type: "chart_bar" }>;
function ChartBarSection({ section, data }: { section: ChartBarSectionT; data: unknown }) {
  const items = getRecordArray(data, section.data_key);
  if (items.length === 0) return null;
  const maxValue = Math.max(...items.map((item) => Number(item[section.y_key]) || 0), 1);
  return (
    <div className="rounded-lg p-4 border space-y-2" style={{ borderColor: "var(--border-color)" }}>
      {items.map((item, i) => {
        const value = Number(item[section.y_key]) || 0;
        const pct = (value / maxValue) * 100;
        return (
          <div key={i} className="flex items-center gap-3 text-xs">
            <div className="w-32 truncate" style={{ color: "var(--text-secondary)" }}>{formatValue(item[section.x_key])}</div>
            <div className="flex-1 h-4 rounded overflow-hidden" style={{ backgroundColor: "var(--bg-tertiary)" }}>
              <div className="h-full" style={{ width: `${pct}%`, backgroundColor: "var(--accent)" }} />
            </div>
            <div className="w-16 text-right" style={{ color: "var(--text-primary)" }}>{formatValue(value)}</div>
          </div>
        );
      })}
    </div>
  );
}

function formatValue(v: unknown): string {
  if (v === null || v === undefined) return "—";
  if (typeof v === "boolean") return v ? "Yes" : "No";
  if (typeof v === "object") return JSON.stringify(v);
  return String(v);
}

function parsePluginUiSchema(value: unknown): PluginUiSchema | null {
  const schema = getRecord(value);
  if (!schema) return null;
  const sections = schema?.sections;
  if (!Array.isArray(sections)) return null;

  const validSections = sections.filter(isUiSection);
  if (validSections.length === 0) return null;

  return {
    title: typeof schema.title === "string" ? schema.title : "Plugin Result",
    sections: validSections,
  };
}

function isUiSection(value: unknown): value is UiSection {
  const section = getRecord(value);
  if (!section || typeof section.type !== "string" || typeof section.data_key !== "string") return false;

  switch (section.type) {
    case "table":
      return Array.isArray(section.columns) && section.columns.every(isTableColumn);
    case "cards":
      return isCardSchema(section.card);
    case "stat_grid":
      return Array.isArray(section.metrics) && section.metrics.every(isMetricSchema);
    case "markdown":
      return true;
    case "chart_bar":
      return typeof section.x_key === "string" && typeof section.y_key === "string";
    default:
      return false;
  }
}

function isTableColumn(value: unknown): value is { key: string; label: string; width?: number } {
  const column = getRecord(value);
  return Boolean(column) && typeof column?.key === "string" && typeof column.label === "string" && (column.width === undefined || typeof column.width === "number");
}

function isCardSchema(value: unknown): value is { title_key: string; body_key: string } {
  const card = getRecord(value);
  return Boolean(card) && typeof card?.title_key === "string" && typeof card.body_key === "string";
}

function isMetricSchema(value: unknown): value is { key: string; label: string; unit?: string } {
  const metric = getRecord(value);
  return Boolean(metric) && typeof metric?.key === "string" && typeof metric.label === "string" && (metric.unit === undefined || typeof metric.unit === "string");
}

function getRecord(value: unknown): JsonRecord | null {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return null;
  return Object.fromEntries(Object.entries(value));
}

function getRecordValue(value: unknown, key: string): unknown {
  return getRecord(value)?.[key];
}

function getRecordArray(value: unknown, key: string): JsonRecord[] {
  const list = getRecordValue(value, key);
  if (!Array.isArray(list)) return [];
  return list.filter((item) => getRecord(item) !== null);
}
