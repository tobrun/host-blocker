export type BlockingMode = "active" | "customPaused" | "allPaused";

export type Extension = "fakenews" | "gambling" | "porn" | "social";

export const EXTENSIONS: Extension[] = ["fakenews", "gambling", "porn", "social"];

/** Wire ids matching the Rust `SourceId` enum (serde renames). */
export type SourceId =
  | "stevenblack"
  | "oisd-small"
  | "adaway"
  | "blocklistproject-phishing";

export interface ListSourceInfo {
  id: SourceId;
  name: string;
  description: string;
  homepage: string;
  /** Category extensions are a StevenBlack-only concept. */
  supportsExtensions: boolean;
}

/** Curated lists offered on the Block Lists page. */
export const SOURCES: ListSourceInfo[] = [
  {
    id: "stevenblack",
    name: "StevenBlack Hosts",
    description:
      "A comprehensive list of ads, trackers and malicious domains. The unified base list is always included; tick categories to extend it.",
    homepage: "https://github.com/StevenBlack/hosts",
    supportsExtensions: true,
  },
  {
    id: "oisd-small",
    name: "OISD Small",
    description:
      "Curated ads/tracker list with a famously low false-positive rate — “Block. Don't break.” Updated daily.",
    homepage: "https://oisd.nl/",
    supportsExtensions: false,
  },
  {
    id: "adaway",
    name: "AdAway",
    description:
      "A small, conservative blocklist of mobile ad and analytics providers. A good light starting point.",
    homepage: "https://adaway.org/",
    supportsExtensions: false,
  },
  {
    id: "blocklistproject-phishing",
    name: "Blocklist Project — Phishing",
    description:
      "Phishing and fraud domains maintained by the Blocklist Project. Security-focused rather than ads.",
    homepage: "https://github.com/blocklistproject/Lists",
    supportsExtensions: false,
  },
];

export function sourceInfo(id: SourceId): ListSourceInfo {
  // Stryker disable next-line All -- the fallback only fires for ids outside
  // the SourceId union, which typed callers (and the backend today) cannot
  // produce, so no test can observe the difference.
  return SOURCES.find((s) => s.id === id) ?? SOURCES[0];
}

export interface BaselineView {
  /** Which curated list is in use. */
  source: SourceId;
  /** e.g. "fakenews-gambling"; null for a plain list. */
  variant: string | null;
  extensions: Extension[];
  hostCount: number;
  fetchedAt: string;
}

/** Human-readable name of the active baseline, e.g. `stevenblack fakenews-gambling`. */
export function baselineLabel(baseline: BaselineView): string {
  const label = sourceInfo(baseline.source).id;
  return baseline.variant ? `${label} ${baseline.variant}` : label;
}

export interface UiState {
  mode: BlockingMode;
  sites: string[];
  baseline: BaselineView | null;
  blockedCount: number;
}

/** Serialised `AppError` from the Rust side. */
export interface ErrorPayload {
  kind: string;
  message: string;
}
