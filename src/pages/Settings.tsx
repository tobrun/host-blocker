import type { UiState } from "../types";

interface Props {
  state: UiState;
  busy: boolean;
  onUpdateBaseline: () => void;
  onRemoveAll: () => void;
}

function formatDate(iso: string): string {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? iso : d.toLocaleString();
}

export default function Settings({ state, busy, onUpdateBaseline, onRemoveAll }: Props) {
  return (
    <>
      <header className="page-header">
        <div>
          <h1>Settings</h1>
          <p>Baseline maintenance and cleanup.</p>
        </div>
      </header>

      <section className="card">
        <div className="card-header">
          <h2>StevenBlack baseline</h2>
        </div>
        <div className="settings-row">
          {state.baseline ? (
            <>
              <div>
                <strong>
                  {state.baseline.variant ? `stevenblack ${state.baseline.variant}` : "stevenblack (unified)"}
                </strong>
                <p className="row-desc">
                  {state.baseline.hostCount} hosts · last updated {formatDate(state.baseline.fetchedAt)}
                </p>
              </div>
              <button type="button" className="btn" disabled={busy} onClick={onUpdateBaseline}>
                Update now
              </button>
            </>
          ) : (
            <p className="row-desc">
              No baseline imported yet. Import one from the Block Lists page.
            </p>
          )}
        </div>
      </section>

      <section className="card">
        <div className="card-header">
          <h2>Cleanup</h2>
        </div>
        <div className="settings-row">
          <div>
            <strong>Remove all HostBlocker entries from /etc/hosts</strong>
            <p className="row-desc">
              Deletes the managed section and resets the app to its defaults.
              The original hosts file backup stays at /etc/hosts.hostblocker.bak.
            </p>
          </div>
          <button
            type="button"
            className="btn danger"
            disabled={busy}
            onClick={() => {
              if (window.confirm("Remove every HostBlocker entry from /etc/hosts?")) {
                onRemoveAll();
              }
            }}
          >
            Remove all entries
          </button>
        </div>
      </section>
    </>
  );
}
