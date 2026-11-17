import { invoke } from "@tauri-apps/api/core";
import type { BlockingMode, Extension, SourceId, UiState } from "./types";

/** Error carrying the backend's user-facing message and machine kind. */
export class ApiError extends Error {
  readonly kind: string;
  constructor(kind: string, message: string) {
    super(message);
    this.kind = kind;
  }
}

function rethrow(err: unknown): never {
  if (typeof err === "object" && err !== null && "message" in err) {
    const payload = err as { kind?: unknown; message?: unknown };
    throw new ApiError(
      typeof payload.kind === "string" ? payload.kind : "error",
      typeof payload.message === "string" ? payload.message : String(err),
    );
  }
  throw new ApiError("error", String(err));
}

export async function getState(): Promise<UiState> {
  return invoke<UiState>("get_state").catch(rethrow);
}

export async function addSite(input: string): Promise<UiState> {
  return invoke<UiState>("add_site", { input }).catch(rethrow);
}

export async function removeSite(input: string): Promise<UiState> {
  return invoke<UiState>("remove_site", { input }).catch(rethrow);
}

export async function importBaseline(
  source: SourceId,
  extensions: Extension[],
): Promise<UiState> {
  return invoke<UiState>("import_baseline", { source, extensions }).catch(rethrow);
}

export async function updateBaseline(): Promise<UiState> {
  return invoke<UiState>("update_baseline").catch(rethrow);
}

export async function removeBaseline(): Promise<UiState> {
  return invoke<UiState>("remove_baseline").catch(rethrow);
}

export async function setMode(mode: BlockingMode): Promise<UiState> {
  return invoke<UiState>("set_mode", { mode }).catch(rethrow);
}

export async function removeAllEntries(): Promise<UiState> {
  return invoke<UiState>("remove_all_entries").catch(rethrow);
}
