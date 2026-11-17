import { useState } from "react";
import { EXTENSIONS, type BaselineView, type Extension } from "../types";

interface Props {
  baseline: BaselineView | null;
  busy: boolean;
  onImport: (extensions: Extension[]) => void;
  onUpdate: () => void;
  onRemove: () => void;
}

export default function StevenBlackCard({ baseline, busy, onImport, onUpdate, onRemove }: Props) {
  const [selected, setSelected] = useState<Extension[]>(baseline?.extensions ?? []);

  const toggle = (ext: Extension) =>
    setSelected((prev) =>
      prev.includes(ext) ? prev.filter((e) => e !== ext) : [...prev, ext],
    );

  const checkboxes = (
    <div className="ext-row" role="group" aria-label="List categories">
      {EXTENSIONS.map((ext) => (
        <label key={ext} className="ext-check">
          <input
            type="checkbox"
            checked={selected.includes(ext)}
            disabled={baseline !== null}
            onChange={() => toggle(ext)}
          />
          {ext}
        </label>
      ))}
    </div>
  );

  return (
    <section className="card">
      <div className="card-header">
        <h2>Popular Lists</h2>
      </div>
      <div className="list-row">
        <div className="row-body">
          <div className="row-title">
            StevenBlack Hosts
            {baseline && (
              <span className="badge">
                {baseline.variant ? `active · ${baseline.variant}` : "active · unified"}
              </span>
            )}
          </div>
          <p className="row-desc">
            A comprehensive list of ads, trackers and malicious domains.
            The unified base list is always included; tick categories to extend it.
          </p>
          <a
            className="row-link"
            href="https://github.com/StevenBlack/hosts"
            onClick={(e) => e.preventDefault()}
            title="Opens in your browser outside the app"
          >
            https://github.com/stevenblack/hosts
          </a>
          {checkboxes}
        </div>
        <div className="row-actions">
          {baseline === null ? (
            <button
              type="button"
              className="btn primary"
              disabled={busy}
              onClick={() => onImport(selected)}
            >
              Import
            </button>
          ) : (
            <>
              <button type="button" className="btn" disabled={busy} onClick={onUpdate}>
                Update
              </button>
              <button type="button" className="btn danger" disabled={busy} onClick={onRemove}>
                Remove
              </button>
            </>
          )}
        </div>
      </div>
    </section>
  );
}
