import type { BlockingMode, UiState } from "../types";

const MODE_LABEL: Record<BlockingMode, string> = {
  active: "Active",
  customPaused: "My list paused",
  allPaused: "All paused",
};

interface Props {
  state: UiState;
  busy: boolean;
  onSetMode: (mode: BlockingMode) => void;
}

export default function StatusCard({ state, busy, onSetMode }: Props) {
  return (
    <section className="status-card" aria-label="Blocking status">
      <div className="status-card-top">
        <div>
          <div className={`status-line status-${state.mode}`}>
            <span className="status-dot" aria-hidden="true" />
            <strong>{MODE_LABEL[state.mode]}</strong>
          </div>
          <div className="status-count">
            {state.blockedCount} host{state.blockedCount === 1 ? "" : "s"} blocked
          </div>
        </div>
        <span className={`status-toggle ${state.mode === "active" ? "on" : "off"}`} aria-hidden="true">
          <span className="knob" />
        </span>
      </div>
      <div className="status-buttons">
        <button
          type="button"
          className="btn small"
          disabled={busy || state.mode === "allPaused"}
          onClick={() => onSetMode("allPaused")}
        >
          Pause all
        </button>
        <button
          type="button"
          className="btn small"
          disabled={busy || state.mode === "customPaused"}
          onClick={() => onSetMode("customPaused")}
        >
          Pause my list
        </button>
        <button
          type="button"
          className="btn small primary"
          disabled={busy || state.mode === "active"}
          onClick={() => onSetMode("active")}
        >
          Resume blocking
        </button>
      </div>
    </section>
  );
}
