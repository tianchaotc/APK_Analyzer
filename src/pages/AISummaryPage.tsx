import { useStore } from "../stores/useStore";
import { PageHeader, Section } from "../components/common/DataTable";
import { confidenceClass } from "../utils/format";
import { Sparkles, Cpu, Code2, AlertTriangle, Zap, Package, ShieldCheck, FileText } from "lucide-react";

export function AISummaryPage() {
  const { analysis } = useStore();
  if (!analysis) return null;
  const ai = analysis.ai_summary;

  if (!ai) {
    return (
      <div>
        <PageHeader title="AI Summary" subtitle="AI-powered analysis and insights" />
        <div className="card flex flex-col items-center py-16">
          <Sparkles size={48} style={{ color: "var(--text-tertiary)" }} />
          <p className="text-sm mt-3" style={{ color: "var(--text-secondary)" }}>AI summary not available</p>
        </div>
      </div>
    );
  }

  return (
    <div>
      <PageHeader title="AI Summary" subtitle="AI-powered analysis and developer insights" />

      {/* Overview */}
      <div className="card mb-6">
        <div className="flex items-start gap-3 mb-3">
          <div className="w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0" style={{ backgroundColor: "var(--accent-subtle)" }}>
            <Sparkles size={20} style={{ color: "var(--accent)" }} />
          </div>
          <div>
            <h2 className="text-sm font-semibold mb-1" style={{ color: "var(--text-secondary)" }}>Overview</h2>
            <p className="text-sm leading-relaxed" style={{ color: "var(--text-primary)" }}>{ai.overview}</p>
          </div>
        </div>
        <div className="flex items-center gap-3 mt-4 pt-4 border-t" style={{ borderColor: "var(--border-subtle)" }}>
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Estimated App Type:</span>
          <span className="badge badge-info">{ai.app_type}</span>
        </div>
      </div>

      {/* Tech stack */}
      <Section title="Technology Stack Detection">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {ai.tech_stack.map((tech, i) => (
            <div key={i} className="card">
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-2">
                  <Code2 size={16} style={{ color: "var(--accent)" }} />
                  <span className="text-sm font-semibold" style={{ color: "var(--text-primary)" }}>{tech.name}</span>
                </div>
                <span className={`badge ${confidenceClass(tech.confidence)}`}>{tech.confidence}</span>
              </div>
              <ul className="space-y-1">
                {tech.evidence.map((ev, j) => (
                  <li key={j} className="text-xs flex items-start gap-1.5" style={{ color: "var(--text-secondary)" }}>
                    <span style={{ color: "var(--accent)" }}>•</span>
                    <span>{ev}</span>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </Section>

      {/* Architecture */}
      <Section title="Architecture Analysis">
        <div className="card flex items-start gap-3">
          <Cpu size={20} style={{ color: "var(--accent)" }} className="flex-shrink-0 mt-0.5" />
          <p className="text-sm leading-relaxed" style={{ color: "var(--text-primary)" }}>{ai.architecture_guess}</p>
        </div>
      </Section>

      {/* Two columns: risks and suggestions */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Potential risks */}
        <Section title="Potential Risks">
          <div className="card">
            {ai.potential_risks.length === 0 ? (
              <div className="flex items-center gap-2 text-sm" style={{ color: "var(--success)" }}>
                <ShieldCheck size={16} /> No significant risks detected
              </div>
            ) : (
              <ul className="space-y-2">
                {ai.potential_risks.map((risk, i) => (
                  <li key={i} className="flex items-start gap-2 text-xs" style={{ color: "var(--text-primary)" }}>
                    <AlertTriangle size={14} style={{ color: "var(--warning)" }} className="flex-shrink-0 mt-0.5" />
                    <span>{risk}</span>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </Section>

        {/* Performance suggestions */}
        <Section title="Performance Suggestions">
          <div className="card">
            <ul className="space-y-2">
              {ai.performance_suggestions.map((sug, i) => (
                <li key={i} className="flex items-start gap-2 text-xs" style={{ color: "var(--text-primary)" }}>
                  <Zap size={14} style={{ color: "var(--accent)" }} className="flex-shrink-0 mt-0.5" />
                  <span>{sug}</span>
                </li>
              ))}
            </ul>
          </div>
        </Section>
      </div>

      {/* Packaging suggestions */}
      <Section title="Packaging Suggestions">
        <div className="card">
          <ul className="space-y-2">
            {ai.packaging_suggestions.map((sug, i) => (
              <li key={i} className="flex items-start gap-2 text-xs" style={{ color: "var(--text-primary)" }}>
                <Package size={14} style={{ color: "var(--accent)" }} className="flex-shrink-0 mt-0.5" />
                <span>{sug}</span>
              </li>
            ))}
          </ul>
        </div>
      </Section>

      {/* Permission review */}
      <Section title="Permission Review">
        <div className="card flex items-start gap-3">
          <FileText size={20} style={{ color: "var(--accent)" }} className="flex-shrink-0 mt-0.5" />
          <p className="text-sm leading-relaxed" style={{ color: "var(--text-primary)" }}>{ai.permission_review}</p>
        </div>
      </Section>
    </div>
  );
}
