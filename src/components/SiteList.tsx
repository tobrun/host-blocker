interface Props {
  sites: string[];
  onRemove: (site: string) => void;
  busy: boolean;
}

export default function SiteList({ sites, onRemove, busy }: Props) {
  return (
    <section className="card">
      <div className="card-header">
        <h2>Your Sites</h2>
      </div>
      {sites.length === 0 ? (
        <p className="empty">No sites yet — add one below.</p>
      ) : (
        <ul className="site-list">
          {sites.map((site) => (
            <li key={site} className="list-row">
              <div className="row-body">
                <span className="site-host">{site}</span>
                <span className="site-lines">4 hosts-file entries</span>
              </div>
              <div className="row-actions">
                <button
                  type="button"
                  className="btn danger small"
                  disabled={busy}
                  onClick={() => onRemove(site)}
                >
                  Remove
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
