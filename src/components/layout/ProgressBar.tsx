import { useStore } from "../../stores/useStore";

export function ProgressBar() {
  const { progress } = useStore();

  if (!progress) return null;

  return (
    <div className="px-4 py-2 border-t flex items-center gap-3" style={{ borderColor: "var(--border-color)", backgroundColor: "var(--bg-secondary)" }}>
      <div className="flex-1">
        <div className="flex items-center justify-between mb-1">
          <span className="text-xs font-medium" style={{ color: "var(--text-secondary)" }}>
            {progress.stage}: {progress.message}
          </span>
          <span className="text-xs" style={{ color: "var(--text-tertiary)" }}>{progress.percent}%</span>
        </div>
        <div className="h-1.5 rounded-full overflow-hidden" style={{ backgroundColor: "var(--bg-tertiary)" }}>
          <div
            className="h-full rounded-full transition-all duration-300"
            style={{ width: `${progress.percent}%`, backgroundColor: "var(--accent)" }}
          />
        </div>
      </div>
    </div>
  );
}
