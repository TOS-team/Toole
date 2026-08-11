export function formatSize(bytes: number): string {
  if (bytes < 1000) return `${bytes} o`;
  if (bytes < 1000 * 1000) return `${(bytes / 1000).toFixed(1)} Ko`;
  if (bytes < 1000 * 1000 * 1000)
    return `${(bytes / (1000 * 1000)).toFixed(1)} Mo`;
  return `${(bytes / (1000 * 1000 * 1000)).toFixed(1)} Go`;
}
