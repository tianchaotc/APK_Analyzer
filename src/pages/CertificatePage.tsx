import { useStore } from "../stores/useStore";
import { PageHeader, Section } from "../components/common/DataTable";
import { copyToClipboard } from "../utils/format";
import { Award, Copy, ShieldCheck, AlertTriangle, Check, X, Clock, Key } from "lucide-react";

export function CertificatePage() {
  const { analysis } = useStore();
  if (!analysis) return null;
  const c = analysis.certificate;

  return (
    <div>
      <PageHeader title="Certificate Analysis" subtitle={c.signature_scheme} />

      {/* Status cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-6">
        <div className="stat-card">
          <ShieldCheck size={18} style={{ color: "var(--accent)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Signature Scheme</span>
          <span className="text-sm font-semibold" style={{ color: "var(--text-primary)" }}>{c.signature_scheme}</span>
        </div>
        <div className="stat-card">
          <Key size={18} style={{ color: c.is_debug_certificate ? "var(--danger)" : "var(--success)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Debug Certificate</span>
          <span className="text-sm font-semibold flex items-center gap-1" style={{ color: c.is_debug_certificate ? "var(--danger)" : "var(--success)" }}>
            {c.is_debug_certificate ? <><X size={14} /> Yes</> : <><Check size={14} /> No</>}
          </span>
        </div>
        <div className="stat-card">
          <Clock size={18} style={{ color: c.is_expired ? "var(--danger)" : "var(--success)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Certificate Status</span>
          <span className="text-sm font-semibold flex items-center gap-1" style={{ color: c.is_expired ? "var(--danger)" : "var(--success)" }}>
            {c.is_expired ? <><AlertTriangle size={14} /> Expired</> : <><Check size={14} /> Valid</>}
          </span>
        </div>
        <div className="stat-card">
          <Award size={18} style={{ color: "var(--accent)" }} />
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>Scheme Support</span>
          <span className="text-sm font-semibold flex gap-1" style={{ color: "var(--text-primary)" }}>
            {c.has_v1 && <span className="badge badge-neutral">V1</span>}
            {c.has_v2 && <span className="badge badge-info">V2</span>}
            {c.has_v3 && <span className="badge badge-success">V3</span>}
          </span>
        </div>
      </div>

      {/* Signers */}
      <Section title={`Signers (${c.signers.length})`}>
        <div className="space-y-4">
          {c.signers.map((signer, i) => (
            <div key={i} className="card">
              <div className="flex items-center gap-3 mb-4">
                <div className="w-10 h-10 rounded-lg flex items-center justify-center" style={{ backgroundColor: "var(--accent-subtle)" }}>
                  <Award size={20} style={{ color: "var(--accent)" }} />
                </div>
                <div className="flex-1">
                  <p className="text-sm font-semibold" style={{ color: "var(--text-primary)" }}>{signer.subject}</p>
                  <p className="text-xs" style={{ color: "var(--text-tertiary)" }}>Valid: {signer.not_before} → {signer.not_after} ({signer.validity_days} days)</p>
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                {[
                  { label: "Subject", value: signer.subject, copyable: true },
                  { label: "Issuer", value: signer.issuer, copyable: true },
                  { label: "Serial Number", value: signer.serial_number, copyable: true },
                  { label: "Public Key Algorithm", value: signer.public_key_algorithm },
                  { label: "Signature Algorithm", value: signer.signature_algorithm },
                  { label: "Validity", value: `${signer.validity_days} days`, badge: signer.is_expired ? { text: "Expired", class: "badge-danger" } : { text: "Valid", class: "badge-success" } },
                ].map((field) => (
                  <div key={field.label} className="p-3 rounded-lg" style={{ backgroundColor: "var(--bg-secondary)" }}>
                    <div className="flex items-center justify-between mb-1">
                      <span className="text-xs font-medium" style={{ color: "var(--text-tertiary)" }}>{field.label}</span>
                      {field.copyable && (
                        <button onClick={() => copyToClipboard(field.value)} className="opacity-50 hover:opacity-100 transition-opacity">
                          <Copy size={12} style={{ color: "var(--text-tertiary)" }} />
                        </button>
                      )}
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-mono break-all" style={{ color: "var(--text-primary)" }}>{field.value}</span>
                      {field.badge && <span className={`badge ${field.badge.class}`}>{field.badge.text}</span>}
                    </div>
                  </div>
                ))}
              </div>

              {/* Hashes */}
              <div className="mt-3 space-y-2">
                {[
                  { label: "SHA-256", value: signer.sha256, color: "var(--accent)" },
                  { label: "SHA-1", value: signer.sha1, color: "var(--warning)" },
                  { label: "MD5", value: signer.md5, color: "var(--danger)" },
                ].map((hash) => (
                  <div key={hash.label} className="flex items-center gap-3 p-2 rounded-lg" style={{ backgroundColor: "var(--bg-secondary)" }}>
                    <span className="badge badge-neutral w-20 justify-center">{hash.label}</span>
                    <span className="text-xs font-mono flex-1 break-all" style={{ color: "var(--text-primary)" }}>{hash.value}</span>
                    <button onClick={() => copyToClipboard(hash.value)} className="opacity-50 hover:opacity-100 transition-opacity">
                      <Copy size={12} style={{ color: "var(--text-tertiary)" }} />
                    </button>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </Section>
    </div>
  );
}
