import { useState } from "react";
import { useStore } from "../stores/useStore";
import { PageHeader, Section } from "../components/common/DataTable";
import { severityClass } from "../utils/format";
import { AlertTriangle, ShieldCheck, AlertOctagon, AlertCircle, Info, CheckCircle2 } from "lucide-react";

export function SecurityPage() {
  const { analysis } = useStore();
  if (!analysis) return null;
  const s = analysis.security;
  const [filter, setFilter] = useState("all");

  const scoreColor = s.score >= 80 ? "var(--success)" : s.score >= 60 ? "var(--warning)" : "var(--danger)";
  const scoreLabel = s.score >= 80 ? "Good" : s.score >= 60 ? "Fair" : "Poor";

  const filters = [
    { id: "all", label: "All" },
    { id: "critical", label: "Critical" },
    { id: "high", label: "High" },
    { id: "medium", label: "Medium" },
    { id: "low", label: "Low" },
    { id: "info", label: "Info" },
  ];

  const filtered = filter === "all" ? s.issues : s.issues.filter(i => i.severity === filter);

  const severityIcon = (severity: string) => {
    switch (severity) {
      case "critical": return <AlertOctagon size={18} style={{ color: "var(--danger)" }} />;
      case "high": return <AlertTriangle size={18} style={{ color: "var(--danger)" }} />;
      case "medium": return <AlertCircle size={18} style={{ color: "var(--warning)" }} />;
      case "low": return <Info size={18} style={{ color: "var(--accent)" }} />;
      case "info": return <Info size={18} style={{ color: "var(--text-tertiary)" }} />;
      default: return <Info size={18} />;
    }
  };

  const issueCounts = {
    critical: s.issues.filter(i => i.severity === "critical").length,
    high: s.issues.filter(i => i.severity === "high").length,
    medium: s.issues.filter(i => i.severity === "medium").length,
    low: s.issues.filter(i => i.severity === "low").length,
    info: s.issues.filter(i => i.severity === "info").length,
  };

  return (
    <div>
      <PageHeader title="Security Analysis" subtitle="Automated security checks and recommendations" />

      {/* Score card */}
      <div className="card mb-6 flex items-center gap-6">
        <div className="relative w-32 h-32 flex-shrink-0">
          <svg className="w-full h-full -rotate-90" viewBox="0 0 120 120">
            <circle cx="60" cy="60" r="50" fill="none" stroke="var(--bg-tertiary)" strokeWidth="10" />
            <circle
              cx="60" cy="60" r="50" fill="none"
              stroke={scoreColor}
              strokeWidth="10"
              strokeLinecap="round"
              strokeDasharray={`${(s.score / 100) * 314.16} 314.16`}
              className="transition-all duration-1000"
            />
          </svg>
          <div className="absolute inset-0 flex flex-col items-center justify-center">
            <span className="text-3xl font-bold" style={{ color: scoreColor }}>{s.score}</span>
            <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>/ 100</span>
          </div>
        </div>
        <div className="flex-1">
          <h2 className="text-xl font-bold mb-1" style={{ color: scoreColor }}>{scoreLabel} Security Score</h2>
          <p className="text-sm mb-3" style={{ color: "var(--text-secondary)" }}>
            {s.score >= 80 ? "This APK follows good security practices with minimal issues." :
             s.score >= 60 ? "This APK has some security concerns that should be addressed." :
             "This APK has significant security issues that need immediate attention."}
          </p>
          <div className="flex gap-4">
            {[
                { label: "Critical", count: issueCounts.critical, color: "var(--danger)" },
                { label: "High", count: issueCounts.high, color: "var(--danger)" },
                { label: "Medium", count: issueCounts.medium, color: "var(--warning)" },
                { label: "Low", count: issueCounts.low, color: "var(--accent)" },
                { label: "Info", count: issueCounts.info, color: "var(--text-tertiary)" },
            ].map((item) => (
              <div key={item.label} className="flex items-center gap-1.5">
                <span className="w-2 h-2 rounded-full" style={{ backgroundColor: item.color }} />
                <span className="text-xs" style={{ color: "var(--text-secondary)" }}>{item.count} {item.label}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Issues */}
      <Section title={`Security Issues (${s.issues.length})`}>
        <div className="flex gap-1 mb-4 p-1 rounded-lg inline-flex" style={{ backgroundColor: "var(--bg-tertiary)" }}>
          {filters.map((f) => (
            <button
              key={f.id}
              onClick={() => setFilter(f.id)}
              className={`px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${filter === f.id ? "text-white" : ""}`}
              style={filter === f.id ? { backgroundColor: "var(--accent)" } : { color: "var(--text-secondary)" }}
            >
              {f.label}
            </button>
          ))}
        </div>

        {filtered.length === 0 ? (
          <div className="card flex flex-col items-center py-12">
            <CheckCircle2 size={48} style={{ color: "var(--success)" }} />
            <p className="text-sm font-medium mt-3" style={{ color: "var(--text-primary)" }}>No issues in this category</p>
          </div>
        ) : (
          <div className="space-y-2">
            {filtered.map((issue, i) => (
              <div key={i} className="card flex items-start gap-3">
                <div className="mt-0.5">{severityIcon(issue.severity)}</div>
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-sm font-semibold" style={{ color: "var(--text-primary)" }}>{issue.title}</span>
                    <span className={`badge ${severityClass(issue.severity)}`}>{issue.severity}</span>
                    <span className="badge badge-neutral">{issue.category}</span>
                  </div>
                  <p className="text-xs mb-2" style={{ color: "var(--text-secondary)" }}>{issue.description}</p>
                  <div className="flex items-start gap-2 p-2 rounded" style={{ backgroundColor: "var(--bg-secondary)" }}>
                    <ShieldCheck size={13} style={{ color: "var(--success)" }} className="mt-0.5 flex-shrink-0" />
                    <span className="text-xs" style={{ color: "var(--text-primary)" }}>{issue.recommendation}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </Section>

      {/* Recommendations */}
      {s.recommendations.length > 0 && (
        <Section title="Recommendations">
          <div className="card">
            <ul className="space-y-2">
              {s.recommendations.map((rec, i) => (
                <li key={i} className="flex items-start gap-2 text-xs" style={{ color: "var(--text-primary)" }}>
                  <span className="text-xs" style={{ color: "var(--accent)" }}>→</span>
                  <span>{rec}</span>
                </li>
              ))}
            </ul>
          </div>
        </Section>
      )}
    </div>
  );
}
