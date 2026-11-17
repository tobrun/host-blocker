export type BlockingMode = "active" | "customPaused" | "allPaused";

export type Extension = "fakenews" | "gambling" | "porn" | "social";

export const EXTENSIONS: Extension[] = ["fakenews", "gambling", "porn", "social"];

export interface BaselineView {
  /** e.g. "fakenews-gambling"; null for the plain unified list. */
  variant: string | null;
  extensions: Extension[];
  hostCount: number;
  fetchedAt: string;
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
