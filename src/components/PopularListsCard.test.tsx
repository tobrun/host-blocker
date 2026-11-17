import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import PopularListsCard from "./PopularListsCard";
import type { BaselineView } from "../types";

const stevenblack: BaselineView = {
  source: "stevenblack",
  variant: "fakenews-gambling",
  extensions: ["fakenews", "gambling"],
  hostCount: 60500,
  fetchedAt: "2026-11-01T10:00:00Z",
};

const oisd: BaselineView = {
  source: "oisd-small",
  variant: null,
  extensions: [],
  hostCount: 40000,
  fetchedAt: "2026-11-01T10:00:00Z",
};

type Handlers = Partial<Record<"onImport" | "onUpdate" | "onRemove", () => void>>;

function renderCard(base: BaselineView | null = null, handlers: Handlers = {}) {
  const onImport = vi.fn(handlers.onImport ?? (() => {}));
  const onUpdate = vi.fn(handlers.onUpdate ?? (() => {}));
  const onRemove = vi.fn(handlers.onRemove ?? (() => {}));
  render(
    <PopularListsCard
      baseline={base}
      busy={false}
      onImport={onImport}
      onUpdate={onUpdate}
      onRemove={onRemove}
    />,
  );
  return { onImport, onUpdate, onRemove };
}

describe("PopularListsCard", () => {
  it("offers every curated list for import", () => {
    renderCard();
    for (const name of [
      "Import StevenBlack Hosts",
      "Import OISD Small",
      "Import AdAway",
      "Import Blocklist Project — Phishing",
    ]) {
      expect(screen.getByRole("button", { name })).toBeTruthy();
    }
  });

  it("imports the plain unified stevenblack list when no categories are ticked", () => {
    const { onImport } = renderCard();
    screen.getByRole("button", { name: "Import StevenBlack Hosts" }).click();
    expect(onImport).toHaveBeenCalledTimes(1);
    expect(onImport).toHaveBeenCalledWith("stevenblack", []);
  });

  it("imports extension-less sources without a selection", () => {
    const { onImport } = renderCard();
    fireEvent.click(screen.getByLabelText("porn"));
    screen.getByRole("button", { name: "Import OISD Small" }).click();
    expect(onImport).toHaveBeenCalledWith("oisd-small", []);
  });

  it("keeps checkboxes editable while no baseline exists", () => {
    renderCard();
    expect((screen.getByLabelText("porn") as HTMLInputElement).disabled).toBe(false);
  });

  it("prevents in-app navigation on every project link", () => {
    renderCard();
    for (const link of screen.getAllByRole("link")) {
      expect(fireEvent.click(link)).toBe(false);
    }
  });

  it("ticking categories passes them to onImport", () => {
    const { onImport } = renderCard();
    fireEvent.click(screen.getByLabelText("gambling"));
    fireEvent.click(screen.getByLabelText("porn"));
    fireEvent.click(screen.getByLabelText("gambling"));
    fireEvent.click(screen.getByLabelText("porn"));
    fireEvent.click(screen.getByLabelText("social"));
    screen.getByRole("button", { name: "Import StevenBlack Hosts" }).click();
    expect(onImport).toHaveBeenCalledTimes(1);
    expect(onImport).toHaveBeenCalledWith("stevenblack", ["social"]);
  });

  it("shows the active variant badge and update/remove once imported", () => {
    const { onUpdate, onRemove } = renderCard(stevenblack);
    expect(screen.getByText("active · fakenews-gambling")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Import StevenBlack Hosts" })).toBeNull();

    screen.getByRole("button", { name: "Update" }).click();
    screen.getByRole("button", { name: "Remove" }).click();
    expect(onUpdate).toHaveBeenCalledTimes(1);
    expect(onRemove).toHaveBeenCalledTimes(1);
  });

  it("lets another list replace the active one", () => {
    const { onImport } = renderCard(stevenblack);
    const replace = screen.getByRole("button", { name: "Replace with AdAway" });
    replace.click();
    expect(onImport).toHaveBeenCalledWith("adaway", []);
    // The active row itself must not offer Replace.
    expect(screen.queryByRole("button", { name: "Replace with StevenBlack Hosts" })).toBeNull();
  });

  it("locks the category checkboxes once stevenblack is the active baseline", () => {
    renderCard(stevenblack);
    for (const ext of ["fakenews", "gambling", "porn", "social"]) {
      const checkbox = screen.getByLabelText(ext) as HTMLInputElement;
      expect(checkbox.disabled).toBe(true);
    }
    expect((screen.getByLabelText("fakenews") as HTMLInputElement).checked).toBe(true);
    expect((screen.getByLabelText("social") as HTMLInputElement).checked).toBe(false);
  });

  it("shows a plain active badge for extension-less sources", () => {
    renderCard(oisd);
    expect(screen.getByText("active")).toBeTruthy();
    expect(screen.queryByText(/^active · /)).toBeNull();
    // StevenBlack is no longer active, so its checkboxes are editable again.
    expect((screen.getByLabelText("porn") as HTMLInputElement).disabled).toBe(false);
  });

  it("shows a plain active badge for stevenblack without a variant", () => {
    renderCard({ ...stevenblack, variant: null, extensions: [] });
    expect(screen.getByText("active")).toBeTruthy();
    expect(screen.queryByText(/^active · /)).toBeNull();
  });

  it("labels the row buttons Import / Replace in plain text", () => {
    // Accessible names come from aria-label, so pin the visible text too.
    const { rerender } = render(
      <PopularListsCard baseline={null} busy={false} onImport={() => {}} onUpdate={() => {}} onRemove={() => {}} />,
    );
    expect(screen.getByRole("button", { name: "Import AdAway" }).textContent).toBe("Import");

    rerender(
      <PopularListsCard
        baseline={oisd}
        busy={false}
        onImport={() => {}}
        onUpdate={() => {}}
        onRemove={() => {}}
      />,
    );
    expect(
      screen.getByRole("button", { name: "Replace with StevenBlack Hosts" }).textContent,
    ).toBe("Replace");
  });
});
