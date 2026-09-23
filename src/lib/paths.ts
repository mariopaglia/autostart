export function shortenPath(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts.length > 3 ? `…\\${parts.slice(-2).join("\\")}` : path;
}
