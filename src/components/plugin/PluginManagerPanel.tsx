import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import type { PluginSummary } from "../../types";
import {
  RefreshCw, FolderOpen, Power, AlertTriangle, CheckCircle2, XCircle, Puzzle,
} from "lucide-react";

/// 插件管理面板。
/// 列出所有已发现插件，支持启用/禁用、查看错误信息、打开插件目录。
/// 通过 Tauri 命令 list_plugins / set_plugin_enabled / get_plugins_dir 与后端交互。
export function PluginManagerPanel() {
  const [plugins, setPlugins] = useState<PluginSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [togglingId, setTogglingId] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const list = await invoke<PluginSummary[]>("list_plugins");
      setPlugins(list);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const handleToggle = async (plugin: PluginSummary, enabled: boolean) => {
    setTogglingId(plugin.id);
    try {
      await invoke("set_plugin_enabled", { pluginId: plugin.id, enabled });
      setPlugins((prev) =>
        prev.map((p) => (p.id === plugin.id ? { ...p, enabled } : p))
      );
    } catch (e) {
      setError(`Failed to toggle ${plugin.id}: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setTogglingId(null);
    }
  };

  const handleOpenDir = async () => {
    try {
      const dir = await invoke<string>("get_plugins_dir");
      await open(dir);
    } catch (e) {
      setError(`Failed to open plugins directory: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const stats = {
    total: plugins.length,
    enabled: plugins.filter((p) => p.enabled).length,
    failed: plugins.filter((p) => p.load_error).length,
  };

  return (
    <div className="space-y-4">
      {/* Header: stats + actions */}
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-3">
          <StatPill label="Total" value={stats.total} color="var(--text-secondary)" />
          <StatPill label="Enabled" value={stats.enabled} color="var(--success)" />
          <StatPill label="Failed" value={stats.failed} color={stats.failed > 0 ? "var(--danger)" : "var(--text-tertiary)"} />
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={refresh}
            disabled={loading}
            className="btn btn-secondary text-xs"
            title="Refresh plugin list"
          >
            <RefreshCw size={14} className={loading ? "animate-spin" : ""} />
            <span>Refresh</span>
          </button>
          <button
            onClick={handleOpenDir}
            className="btn btn-secondary text-xs"
            title="Open plugins directory in file manager"
          >
            <FolderOpen size={14} />
            <span>Open Directory</span>
          </button>
        </div>
      </div>

      {error && (
        <div className="rounded-lg p-3 border flex items-start gap-2" style={{ borderColor: "var(--danger)", backgroundColor: "var(--danger-bg)" }}>
          <AlertTriangle size={15} color="var(--danger)" className="mt-0.5 flex-shrink-0" />
          <span className="text-xs" style={{ color: "var(--text-primary)" }}>{error}</span>
        </div>
      )}

      <div className="rounded-lg p-3 border text-xs" style={{ borderColor: "var(--border-subtle)", backgroundColor: "var(--bg-secondary)", color: "var(--text-tertiary)" }}>
        Refresh reloads the currently registered plugin list. If you add or replace plugin files, restart the app or re-run analysis if the new plugin does not appear.
      </div>

      {/* Plugin list */}
      {plugins.length === 0 && !loading ? (
        <div className="card flex flex-col items-center py-12">
          <Puzzle size={42} style={{ color: "var(--text-tertiary)" }} />
          <p className="text-sm font-medium mt-3" style={{ color: "var(--text-primary)" }}>No plugins installed</p>
          <p className="text-xs mt-1 text-center" style={{ color: "var(--text-tertiary)" }}>
            Place plugin directories under the plugins directory.<br />
            Each plugin should contain <code className="px-1 rounded" style={{ backgroundColor: "var(--bg-tertiary)" }}>manifest.json</code> and a <code className="px-1 rounded" style={{ backgroundColor: "var(--bg-tertiary)" }}>plugin.dylib/.dll/.so</code> file.
          </p>
          <button onClick={handleOpenDir} className="btn btn-secondary text-xs mt-4">
            <FolderOpen size={14} />
            <span>Open Plugins Directory</span>
          </button>
        </div>
      ) : (
        <div className="space-y-2">
          {plugins.map((p) => (
            <PluginRow
              key={p.id}
              plugin={p}
              toggling={togglingId === p.id}
              onToggle={(enabled) => handleToggle(p, enabled)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function StatPill({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg" style={{ backgroundColor: "var(--bg-tertiary)" }}>
      <span className="text-xs uppercase tracking-wide" style={{ color: "var(--text-tertiary)" }}>{label}</span>
      <span className="text-sm font-semibold" style={{ color }}>{value}</span>
    </div>
  );
}

function PluginRow({
  plugin, toggling, onToggle,
}: {
  plugin: PluginSummary;
  toggling: boolean;
  onToggle: (enabled: boolean) => void;
}) {
  const hasError = plugin.load_error !== null && plugin.load_error !== undefined;
  const statusColor = hasError ? "var(--danger)" : plugin.enabled ? "var(--success)" : "var(--text-tertiary)";
  const statusIcon = hasError ? <XCircle size={13} color={statusColor} /> :
    plugin.enabled ? <CheckCircle2 size={13} color={statusColor} /> :
    <Power size={13} color={statusColor} />;
  const statusLabel = hasError ? "Error" : plugin.enabled ? "Enabled" : "Disabled";

  return (
    <div className="card p-3 flex items-start gap-3">
      <div className="mt-0.5">
        <Puzzle size={18} style={{ color: hasError ? "var(--danger)" : "var(--accent)" }} />
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 flex-wrap">
          <span className="text-sm font-semibold" style={{ color: "var(--text-primary)" }}>{plugin.name || plugin.id}</span>
          <span className="badge badge-neutral">v{plugin.version}</span>
          {plugin.capabilities.map((c) => (
            <span key={c} className="badge badge-neutral">{c}</span>
          ))}
          <span className="flex items-center gap-1 text-xs" style={{ color: statusColor }}>
            {statusIcon}
            {statusLabel}
          </span>
        </div>
        {plugin.id && plugin.id !== plugin.name && (
          <p className="text-xs mt-0.5" style={{ color: "var(--text-tertiary)" }}>{plugin.id}</p>
        )}
        {plugin.description && (
          <p className="text-xs mt-1" style={{ color: "var(--text-secondary)" }}>{plugin.description}</p>
        )}
        {plugin.author && (
          <p className="text-xs mt-0.5" style={{ color: "var(--text-tertiary)" }}>by {plugin.author}</p>
        )}
        {hasError && (
          <div className="mt-2 rounded p-2 text-xs" style={{ backgroundColor: "var(--danger-bg)", color: "var(--danger)" }}>
            <span className="font-semibold">Load error: </span>
            <span className="break-all">{plugin.load_error}</span>
          </div>
        )}
      </div>
      <div className="flex-shrink-0">
        <Toggle
          checked={plugin.enabled && !hasError}
          disabled={hasError || toggling}
          onChange={onToggle}
        />
      </div>
    </div>
  );
}

function Toggle({
  checked, disabled, onChange,
}: {
  checked: boolean;
  disabled: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <button
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className="relative inline-flex h-5 w-9 items-center rounded-full transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
      style={{ backgroundColor: checked ? "var(--accent)" : "var(--bg-tertiary)" }}
    >
      <span
        className={`inline-block h-3.5 w-3.5 rounded-full bg-white transition-transform ${checked ? "translate-x-5" : "translate-x-1"}`}
      />
    </button>
  );
}
