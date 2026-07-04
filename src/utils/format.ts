export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function formatNumber(n: number): string {
  return n.toLocaleString("en-US");
}

export function truncateMiddle(str: string, maxLen: number): string {
  if (str.length <= maxLen) return str;
  const half = Math.floor((maxLen - 3) / 2);
  return str.substring(0, half) + "..." + str.substring(str.length - half);
}

export function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text).catch(() => {});
}

export function severityClass(severity: string): string {
  switch (severity.toLowerCase()) {
    case "critical": return "badge-danger";
    case "high": return "badge-danger";
    case "medium": return "badge-warning";
    case "low": return "badge-success";
    case "info": return "badge-info";
    default: return "badge-neutral";
  }
}

export function riskClass(risk: string): string {
  switch (risk.toLowerCase()) {
    case "critical": return "badge-danger";
    case "high": return "badge-danger";
    case "medium": return "badge-warning";
    case "low": return "badge-success";
    default: return "badge-neutral";
  }
}

export function protectionLevelClass(level: string): string {
  switch (level.toLowerCase()) {
    case "dangerous": return "badge-danger";
    case "special": return "badge-warning";
    case "signature": return "badge-info";
    case "normal": return "badge-success";
    default: return "badge-neutral";
  }
}

export function confidenceClass(confidence: string): string {
  switch (confidence.toLowerCase()) {
    case "high": return "badge-success";
    case "medium": return "badge-warning";
    case "low": return "badge-neutral";
    default: return "badge-neutral";
  }
}
