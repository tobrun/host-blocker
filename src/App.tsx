import { useCallback, useEffect, useState } from "react";
import * as api from "./api";
import { ApiError } from "./api";
import Sidebar, { type Page } from "./components/Sidebar";
import About from "./pages/About";
import BlockLists from "./pages/BlockLists";
import Settings from "./pages/Settings";
import type { BlockingMode, Extension, UiState } from "./types";

export default function App() {
  const [state, setState] = useState<UiState | null>(null);
  const [page, setPage] = useState<Page>("lists");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    api
      .getState()
      .then(setState)
      .catch((e: unknown) =>
        setNotice(e instanceof Error ? e.message : String(e)),
      );
  }, []);

  /**
   * Run a mutation, adopt the returned state, surface errors as a notice.
   * When `rethrow` is set the error is also re-raised so a component (the
   * add-site form) can show it inline instead.
   */
  const run = useCallback(
    async (fn: () => Promise<UiState>, rethrow = false): Promise<void> => {
      setBusy(true);
      setNotice(null);
      try {
        setState(await fn());
      } catch (e) {
        const message =
          e instanceof ApiError && e.kind === "cancelled"
            ? "The administrator prompt was cancelled — nothing was changed."
            : e instanceof Error
              ? e.message
              : String(e);
        if (!rethrow) setNotice(message);
        if (rethrow) {
          throw new ApiError(e instanceof ApiError ? e.kind : "error", message);
        }
      } finally {
        setBusy(false);
      }
    },
    [],
  );

  return (
    <div className="layout">
      <Sidebar
        page={page}
        onNavigate={setPage}
        state={state}
        busy={busy}
        onSetMode={(mode: BlockingMode) => run(() => api.setMode(mode))}
      />
      <main className="content">
        {notice && (
          <div className="notice" role="alert">
            <span>{notice}</span>
            <button type="button" aria-label="Dismiss" onClick={() => setNotice(null)}>
              ✕
            </button>
          </div>
        )}
        {state === null ? (
          <p className="empty">Loading…</p>
        ) : page === "lists" ? (
          <BlockLists
            state={state}
            busy={busy}
            onAddSite={(input) => run(() => api.addSite(input), true)}
            onRemoveSite={(site) => run(() => api.removeSite(site))}
            onImportBaseline={(exts: Extension[]) => run(() => api.importBaseline(exts))}
            onUpdateBaseline={() => run(() => api.updateBaseline())}
            onRemoveBaseline={() => run(() => api.removeBaseline())}
          />
        ) : page === "settings" ? (
          <Settings
            state={state}
            busy={busy}
            onUpdateBaseline={() => run(() => api.updateBaseline())}
            onRemoveAll={() => run(() => api.removeAllEntries())}
          />
        ) : (
          <About />
        )}
      </main>
    </div>
  );
}
