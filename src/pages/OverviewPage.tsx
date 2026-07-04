import { useStore } from "../stores/useStore";
import { PageHeader, InfoGrid, Section } from "../components/common/DataTable";
import { formatFileSize, formatNumber, copyToClipboard } from "../utils/format";
import { Copy, Smartphone, Package, Shield, HardDrive, Globe, Layers, Zap } from "lucide-react";

export function OverviewPage() {
  const { analysis } = useStore();
  if (!analysis) return null;
  const o = analysis.overview;

  const stats = [
    { label: "App Name", value: o.app_name || "Unknown" },
    { label: "Package Name", value: o.package_name || "Unknown" },
    { label: "Version Name", value: o.version_name || "—" },
    { label: "Version Code", value: o.version_code || "—" },
    { label: "Min SDK", value: o.min_sdk || "—" },
    { label: "Target SDK", value: o.target_sdk || "—" },
    { label: "Compile SDK", value: o.compile_sdk || "—" },
    { label: "APK Size", value: formatFileSize(o.apk_size) },
    { label: "Est. Install Size", value: formatFileSize(o.estimated_install_size) },
    { label: "Signature Version", value: analysis.certificate.signature_scheme || "—" },
    { label: "Split APK", value: o.split_apk ? "Yes" : "No" },
    { label: "Instant App", value: o.instant_app ? "Yes" : "No" },
  ];

  const flags = [
    { label: "Debuggable", value: o.debuggable, danger: true },
    { label: "Allow Backup", value: o.allow_backup, warning: true },
    { label: "Extract Native Libs", value: o.extract_native_libs, warning: true },
    { label: "Cleartext Traffic", value: o.uses_cleartext_traffic, danger: true },
  ];

  return (
    <div>
      <PageHeader title="Overview" subtitle="Basic APK information and app metadata" />

      {/* App identity card */}
      <div className="card mb-6 flex items-center gap-5">
        <div className="w-20 h-20 rounded-2xl flex items-center justify-center flex-shrink-0 overflow-hidden" style={{ backgroundColor: "var(--bg-tertiary)" }}>
          {o.app_icon_base64 ? (
            <img src={`data:image/png;base64,${o.app_icon_base64}`} alt="App icon" className="w-full h-full object-cover" />
          ) : (
            <Package size={36} style={{ color: "var(--text-tertiary)" }} />
          )}
        </div>
        <div className="flex-1">
          <h2 className="text-xl font-bold" style={{ color: "var(--text-primary)" }}>{o.app_name || "Unknown App"}</h2>
          <p className="text-sm" style={{ color: "var(--text-secondary)" }}>{o.package_name}</p>
          <div className="flex items-center gap-4 mt-2">
            <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>v{o.version_name} ({o.version_code})</span>
            <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>·</span>
            <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Min SDK {o.min_sdk}</span>
            <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>·</span>
            <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Target SDK {o.target_sdk}</span>
          </div>
        </div>
        <div className="text-right">
          <div className="text-2xl font-bold" style={{ color: "var(--accent)" }}>{formatFileSize(o.apk_size)}</div>
          <div className="text-xs" style={{ color: "var(--text-tertiary)" }}>APK Size</div>
        </div>
      </div>

      <Section title="Basic Information">
        <InfoGrid items={stats} />
      </Section>

      <Section title="Security Flags">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          {flags.map((f) => (
            <div key={f.label} className="stat-card">
              <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>{f.label}</span>
              <span className="text-sm font-semibold flex items-center gap-2" style={{ color: f.value ? (f.danger ? "var(--danger)" : "var(--warning)") : "var(--success)" }}>
                {f.value ? "Enabled" : "Disabled"}
                {f.value && f.danger && <span className="w-2 h-2 rounded-full" style={{ backgroundColor: "var(--danger)" }} />}
                {f.value && f.warning && <span className="w-2 h-2 rounded-full" style={{ backgroundColor: "var(--warning)" }} />}
              </span>
            </div>
          ))}
        </div>
      </Section>

      <Section title="Architecture & Compatibility">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
          <div className="card">
            <div className="flex items-center gap-2 mb-2">
              <Layers size={16} style={{ color: "var(--accent)" }} />
              <span className="text-xs font-medium" style={{ color: "var(--text-secondary)" }}>ABIs</span>
            </div>
            {o.abis.length > 0 ? (
              <div className="flex flex-wrap gap-1.5">
                {o.abis.map((abi) => (
                  <span key={abi} className="badge badge-info">{abi}</span>
                ))}
              </div>
            ) : <span className="text-sm" style={{ color: "var(--text-tertiary)" }}>No native libraries</span>}
          </div>
          <div className="card">
            <div className="flex items-center gap-2 mb-2">
              <Smartphone size={16} style={{ color: "var(--accent)" }} />
              <span className="text-xs font-medium" style={{ color: "var(--text-secondary)" }}>Densities</span>
            </div>
            {o.densities.length > 0 ? (
              <div className="flex flex-wrap gap-1.5">
                {o.densities.map((d) => (
                  <span key={d} className="badge badge-neutral">{d}</span>
                ))}
              </div>
            ) : <span className="text-sm" style={{ color: "var(--text-tertiary)" }}>None detected</span>}
          </div>
          <div className="card">
            <div className="flex items-center gap-2 mb-2">
              <Globe size={16} style={{ color: "var(--accent)" }} />
              <span className="text-xs font-medium" style={{ color: "var(--text-secondary)" }}>Languages</span>
            </div>
            {o.languages.length > 0 ? (
              <div className="flex flex-wrap gap-1.5">
                {o.languages.map((l) => (
                  <span key={l} className="badge badge-neutral">{l}</span>
                ))}
              </div>
            ) : <span className="text-sm" style={{ color: "var(--text-tertiary)" }}>Default only</span>}
          </div>
        </div>
      </Section>

      <Section title="Quick Stats">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          <div className="stat-card">
            <Shield size={18} style={{ color: "var(--accent)" }} />
            <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Security Score</span>
            <span className="text-lg font-bold" style={{ color: analysis.security.score >= 80 ? "var(--success)" : analysis.security.score >= 60 ? "var(--warning)" : "var(--danger)" }}>
              {analysis.security.score}/100
            </span>
          </div>
          <div className="stat-card">
            <Package size={18} style={{ color: "var(--accent)" }} />
            <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Permissions</span>
            <span className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{formatNumber(analysis.permissions.summary.total)}</span>
          </div>
          <div className="stat-card">
            <HardDrive size={18} style={{ color: "var(--accent)" }} />
            <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Native Libs</span>
            <span className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{formatNumber(analysis.native_libs.summary.total)}</span>
          </div>
          <div className="stat-card">
            <Zap size={18} style={{ color: "var(--accent)" }} />
            <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>DEX Classes</span>
            <span className="text-lg font-bold" style={{ color: "var(--text-primary)" }}>{formatNumber(analysis.dex.summary.total_classes)}</span>
          </div>
        </div>
      </Section>
    </div>
  );
}
