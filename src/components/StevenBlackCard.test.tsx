import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import StevenBlackCard from "./StevenBlackCard";
import type { BaselineView } from "../types";

const baseline: BaselineView = {
  variant: "fakenews-gambling",
  extensions: ["fakenews", "gambling"],
  hostCount: 60500,
  fetchedAt: "2026-11-01T10:00:00Z",
};

function renderCard(
  base: BaselineView | null = null,
  handlers: Partial<Record<"onImport" | "onUpdate" | "onRemove", () => void>> = {},
) {
  const onImport = vi.fn(handlers.onImport ?? (() => {}));
  const onUpdate = vi.fn(handlers.onUpdate ?? (() => {}));
  const onRemove = vi.fn(handlers.onRemove ?? (() => {}));
  render(
    <StevenBlackCard
      baseline={base}
      busy={false}
      onImport={onImport}
      onUpdate={onUpdate}
      onRemove={onRemove}
    />,
  );
  return { onImport, onUpdate, onRemove };
}

describe("StevenBlackCard", () => {
  it("imports the plain unified list when no categories are ticked", () => {
    const { onImport } = renderCard();
    screen.getByRole("button", { name: "Import" }).click();
    expect(onImport).toHaveBeenCalledTimes(1);
    expect(onImport).toHaveBeenCalledWith([]);
  });

  it("ticking categories passes them to onImport", () => {
    const { onImport } = renderCard();
    fireEvent.click(screen.getByLabelText("gambling"));
    fireEvent.click(screen.getByLabelText("porn"));
    fireEvent.click(screen.getByLabelText("gambling"));
    fireEvent.click(screen.getByLabelText("porn"));
    fireEvent.click(screen.getByLabelText("social"));
    screen.getByRole("button", { name: "Import" }).click();
    expect(onImport).toHaveBeenCalledTimes(1);
    expect(onImport).toHaveBeenCalledWith(["social"]);
  });

  it("shows the active variant badge and update/remove once imported", () => {
    const { onUpdate, onRemove } = renderCard(baseline);
    expect(screen.getByText("active · fakenews-gambling")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Import" })).toBeNull();

    screen.getByRole("button", { name: "Update" }).click();
    screen.getByRole("button", { name: "Remove" }).click();
    expect(onUpdate).toHaveBeenCalledTimes(1);
    expect(onRemove).toHaveBeenCalledTimes(1);
  });

  it("locks the category checkboxes once a baseline exists", () => {
    renderCard(baseline);
    for (const ext of ["fakenews", "gambling", "porn", "social"]) {
      const checkbox = screen.getByLabelText(ext) as HTMLInputElement;
      expect(checkbox.disabled).toBe(true);
    }
    expect((screen.getByLabelText("fakenews") as HTMLInputElement).checked).toBe(true);
    expect((screen.getByLabelText("social") as HTMLInputElement).checked).toBe(false);
  });

  it("shows the unified badge when the baseline has no variant", () => {
    renderCard({ ...baseline, variant: null, extensions: [] });
    expect(screen.getByText("active · unified")).toBeTruthy();
  });
});
