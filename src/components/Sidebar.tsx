import StatusCard from "./StatusCard";
import type { BlockingMode, UiState } from "../types";

export type Page = "lists" | "settings" | "about";

const NAV: Array<{ id: Page; label: string }> = [
  { id: "lists", label: "Block Lists" },
  { id: "settings", label: "Settings" },
  { id: "about", label: "About" },
];

interface Props {
  page: Page;
  onNavigate: (page: Page) => void;
  state: UiState | null;
  busy: boolean;
  onSetMode: (mode: BlockingMode) => void;
}

export default function Sidebar({ page, onNavigate, state, busy, onSetMode }: Props) {
  return (
    <aside className="sidebar">
      <div className="brand">
        <div className="brand-logo" aria-hidden="true">
          <svg viewBox="0 0 24 24" width="56" height="56" fill="none">
            <path
              d="M12 2 4 5v6c0 5 3.4 9 8 11 4.6-2 8-6 8-11V5l-8-3Z"
              fill="currentColor"
            />
            <circle cx="12" cy="10.5" r="5" stroke="#fff" strokeWidth="1.4" fill="none" />
            <path
              d="M7 10.5h10M12 5.5c-1.7 1.5-1.7 8.5 0 10M12 5.5c1.7 1.5 1.7 8.5 0 10"
              stroke="#fff"
              strokeWidth="1.4"
              fill="none"
            />
          </svg>
        </div>
        <h1>HostBlocker</h1>
        <p>Block websites by blocking DNS resolution</p>
      </div>
      <nav className="nav">
        {NAV.map((item) => (
          <button
            key={item.id}
            type="button"
            className={`nav-item ${page === item.id ? "active" : ""}`}
            aria-current={page === item.id ? "page" : undefined}
            onClick={() => onNavigate(item.id)}
          >
            {item.label}
          </button>
        ))}
      </nav>
      <div className="sidebar-footer">
        {state ? (
          <StatusCard state={state} busy={busy} onSetMode={onSetMode} />
        ) : (
          <div className="status-card placeholder">Loading status…</div>
        )}
      </div>
    </aside>
  );
}
