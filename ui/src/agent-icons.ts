const files = import.meta.glob("../assets/agent-icons/*.{svg,png}", {
  eager: true,
  query: "?url",
  import: "default",
}) as Record<string, string>;

const urls = new Map(
  Object.entries(files).map(([path, url]) => [
    path.split("/").pop()!.replace(/\.(svg|png)$/, ""),
    url,
  ]),
);

/** Resolve one bundled vendor icon, or undefined when the registry uses none. */
export function agentIconUrl(name: string) {
  return urls.get(name);
}
