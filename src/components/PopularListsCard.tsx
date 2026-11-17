import { useState } from "react";
import {
  EXTENSIONS,
  SOURCES,
  type BaselineView,
  type Extension,
  type SourceId,
} from "../types";

interface Props {
  baseline: BaselineView | null;
  busy: boolean;
  onImport: (source: SourceId, extensions: Extension[]) => void;
  onUpdate: () => void;
  onRemove: () => void;
}

export default function PopularListsCard({
  baseline,
  busy,
  onImport,
  onUpdate,
  onRemove,
}: Props) {
  const [selected, setSelected] = useState<Extension[]>(baseline?.extensions ?? []);

  const toggle = (ext: Extension) =>
    setSelected((prev) =>
      prev.includes(ext) ? prev.filter((e) => e !== ext) : [...prev, ext],
    );

  return (
    <section className="card">
      <div className="card-header">
        <h2>Popular Lists</h2>
      </div>
      {SOURCES.map((source) => {
        const active = baseline?.source === source.id;
        const locked = active;
        return (
          <div className="list-row" key={source.id}>
            <div className="row-body">
              <div className="row-title">
                {source.name}
                {active && (
                  <span className="badge">
                    {source.supportsExtensions && baseline.variant
                      ? `active · ${baseline.variant}`
                      : "active"}
                  </span>
                )}
              </div>
              <p className="row-desc">{source.description}</p>
              <a
                className="row-link"
                href={source.homepage}
                onClick={(e) => e.preventDefault()}
                title="Opens in your browser outside the app"
              >
                {source.homepage}
              </a>
              {source.supportsExtensions && (
                <div className="ext-row" role="group" aria-label="List categories">
                  {EXTENSIONS.map((ext) => (
                    <label key={ext} className="ext-check">
                      <input
                        type="checkbox"
                        checked={selected.includes(ext)}
                        disabled={locked}
                        onChange={() => toggle(ext)}
                      />
                      {ext}
                    </label>
                  ))}
                </div>
              )}
            </div>
            <div className="row-actions">
              {active ? (
                <>
                  <button
                    type="button"
                    className="btn"
                    disabled={busy}
                    onClick={onUpdate}
                  >
                    Update
                  </button>
                  <button
                    type="button"
                    className="btn danger"
                    disabled={busy}
                    onClick={onRemove}
                  >
                    Remove
                  </button>
                </>
              ) : (
                <button
                  type="button"
                  className="btn primary"
                  disabled={busy}
                  aria-label={`${baseline === null ? "Import" : "Replace with"} ${source.name}`}
                  onClick={() =>
                    onImport(source.id, source.supportsExtensions ? selected : [])
                  }
                >
                  {baseline === null ? "Import" : "Replace"}
                </button>
              )}
            </div>
          </div>
        );
      })}
    </section>
  );
}
