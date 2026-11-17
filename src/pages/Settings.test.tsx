import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import Settings from "./Settings";
import type { UiState } from "../types";

function uiState(overrides: Partial<UiState> = {}): UiState {
  return {
    mode: "active",
    sites: [],
    baseline: {
      variant: "gambling",
      extensions: ["gambling"],
      hostCount: 60500,
      fetchedAt: "2026-11-01T10:00:00Z",
    },
    blockedCount: 0,
    ...overrides,
  };
}

describe("Settings", () => {
  it("describes the installed variant, count and date", () => {
    render(
      <Settings state={uiState()} busy={false} onUpdateBaseline={() => {}} onRemoveAll={() => {}} />,
    );
    expect(screen.getByText("stevenblack gambling")).toBeTruthy();
    expect(screen.getByText(/60500 hosts · last updated/)).toBeTruthy();
  });

  it("shows the unified label when there is no variant", () => {
    render(
      <Settings
        state={uiState({ baseline: { variant: null, extensions: [], hostCount: 59000, fetchedAt: "2026-11-01T10:00:00Z" } })}
        busy={false}
        onUpdateBaseline={() => {}}
        onRemoveAll={() => {}}
      />,
    );
    expect(screen.getByText("stevenblack (unified)")).toBeTruthy();
  });

  it("falls back to the raw timestamp for unparseable dates", () => {
    render(
      <Settings
        state={uiState({ baseline: { variant: null, extensions: [], hostCount: 1, fetchedAt: "garbage" } })}
        busy={false}
        onUpdateBaseline={() => {}}
        onRemoveAll={() => {}}
      />,
    );
    expect(screen.getByText(/last updated garbage/)).toBeTruthy();
  });

  it("points at the Block Lists page when no baseline exists", () => {
    render(
      <Settings
        state={uiState({ baseline: null })}
        busy={false}
        onUpdateBaseline={() => {}}
        onRemoveAll={() => {}}
      />,
    );
    expect(screen.getByText(/No baseline imported yet/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Update now" })).toBeNull();
  });

  it("updates the baseline on click", () => {
    const onUpdateBaseline = vi.fn();
    render(
      <Settings state={uiState()} busy={false} onUpdateBaseline={onUpdateBaseline} onRemoveAll={() => {}} />,
    );
    screen.getByRole("button", { name: "Update now" }).click();
    expect(onUpdateBaseline).toHaveBeenCalledTimes(1);
  });

  it("asks for confirmation before removing all entries", () => {
    const onRemoveAll = vi.fn();
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(false);
    render(
      <Settings state={uiState()} busy={false} onUpdateBaseline={() => {}} onRemoveAll={onRemoveAll} />,
    );
    screen.getByRole("button", { name: "Remove all entries" }).click();
    expect(confirmSpy).toHaveBeenCalledTimes(1);
    expect(confirmSpy).toHaveBeenCalledWith(
      "Remove every HostBlocker entry from /etc/hosts?",
    );
    expect(onRemoveAll).not.toHaveBeenCalled();

    confirmSpy.mockReturnValue(true);
    screen.getByRole("button", { name: "Remove all entries" }).click();
    expect(onRemoveAll).toHaveBeenCalledTimes(1);
    confirmSpy.mockRestore();
  });

  it("disables both buttons while busy", () => {
    render(
      <Settings state={uiState()} busy onUpdateBaseline={() => {}} onRemoveAll={() => {}} />,
    );
    expect(screen.getByRole("button", { name: "Update now" }).hasAttribute("disabled")).toBe(true);
    expect(screen.getByRole("button", { name: "Remove all entries" }).hasAttribute("disabled")).toBe(true);
  });
});
