import { useState } from "react";
import AddSiteForm from "../components/AddSiteForm";
import PopularListsCard from "../components/PopularListsCard";
import SiteList from "../components/SiteList";
import type { Extension, SourceId, UiState } from "../types";

interface Props {
  state: UiState;
  busy: boolean;
  onAddSite: (input: string) => Promise<void>;
  onRemoveSite: (site: string) => void;
  onImportBaseline: (source: SourceId, extensions: Extension[]) => void;
  onUpdateBaseline: () => void;
  onRemoveBaseline: () => void;
}

export default function BlockLists({
  state,
  busy,
  onAddSite,
  onRemoveSite,
  onImportBaseline,
  onUpdateBaseline,
  onRemoveBaseline,
}: Props) {
  const [query, setQuery] = useState("");
  const filtered = state.sites.filter((s) => s.includes(query.trim().toLowerCase()));

  return (
    <>
      <header className="page-header">
        <div>
          <h1>Block Lists</h1>
          <p>Import a curated baseline or add your own sites, one URL at a time.</p>
        </div>
        <input
          type="search"
          className="search"
          placeholder="Search your sites…"
          aria-label="Search your sites"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
      </header>
      <PopularListsCard
        baseline={state.baseline}
        busy={busy}
        onImport={onImportBaseline}
        onUpdate={onUpdateBaseline}
        onRemove={onRemoveBaseline}
      />
      <SiteList sites={filtered} onRemove={onRemoveSite} busy={busy} />
      <AddSiteForm busy={busy} onSubmit={onAddSite} />
    </>
  );
}
