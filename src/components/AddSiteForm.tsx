import { useState, type FormEvent } from "react";

interface Props {
  busy: boolean;
  onSubmit: (input: string) => Promise<void>;
}

export default function AddSiteForm({ busy, onSubmit }: Props) {
  const [value, setValue] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    const input = value.trim();
    if (!input || pending) return;
    setPending(true);
    setError(null);
    try {
      await onSubmit(input);
      setValue("");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setPending(false);
    }
  };

  return (
    <section className="card add-site">
      <div className="row-body">
        <h2>Add a site</h2>
        <p className="row-desc">
          Paste any page URL (e.g. https://www.reddit.com/r/all) — HostBlocker blocks
          the whole site, with and without www.
        </p>
        <form className="add-site-form" onSubmit={submit}>
          <input
            type="text"
            aria-label="Website URL"
            placeholder="https://www.example.com/some/page"
            value={value}
            onChange={(e) => setValue(e.target.value)}
          />
          <button type="submit" className="btn primary" disabled={busy || pending || !value.trim()}>
            {pending ? "Applying…" : "Add"}
          </button>
        </form>
        {error && (
          <p className="form-error" role="alert">
            {error}
          </p>
        )}
      </div>
    </section>
  );
}
