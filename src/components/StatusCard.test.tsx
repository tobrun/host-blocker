import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import StatusCard from "./StatusCard";
import type { UiState } from "../types";

function uiState(overrides: Partial<UiState> = {}): UiState {
  return {
    mode: "active",
    sites: [],
    baseline: null,
    blockedCount: 12,
    ...overrides,
  };
}

function renderCard(state: UiState = uiState(), busy = false) {
  const onSetMode = vi.fn();
  render(<StatusCard state={state} busy={busy} onSetMode={onSetMode} />);
  return onSetMode;
}

describe("StatusCard", () => {
  it("shows the active mode and the blocked host count", () => {
    renderCard();
    expect(screen.getByText("Active")).toBeTruthy();
    expect(screen.getByText("12 hosts blocked")).toBeTruthy();
    expect(screen.queryByText("1 host blocked")).toBeNull();
  });

  it("singularises a single blocked host", () => {
    renderCard(uiState({ blockedCount: 1 }));
    expect(screen.getByText("1 host blocked")).toBeTruthy();
  });

  it("labels the custom-paused mode", () => {
    renderCard(uiState({ mode: "customPaused" }));
    expect(screen.getByText("My list paused")).toBeTruthy();
  });

  it("labels the all-paused mode", () => {
    renderCard(uiState({ mode: "allPaused" }));
    expect(screen.getByText("All paused")).toBeTruthy();
  });

  it("disables the button matching the active mode", () => {
    renderCard();
    expect(screen.getByRole("button", { name: "Resume blocking" }).hasAttribute("disabled")).toBe(true);
    expect(screen.getByRole("button", { name: "Pause all" }).hasAttribute("disabled")).toBe(false);
    expect(screen.getByRole("button", { name: "Pause my list" }).hasAttribute("disabled")).toBe(false);
  });

  it("enables resume and disables pause-all when fully paused", () => {
    renderCard(uiState({ mode: "allPaused" }));
    expect(screen.getByRole("button", { name: "Pause all" }).hasAttribute("disabled")).toBe(true);
    expect(screen.getByRole("button", { name: "Resume blocking" }).hasAttribute("disabled")).toBe(false);
  });

  it("enables pause-all and disables pause-my-list when the custom list is paused", () => {
    renderCard(uiState({ mode: "customPaused" }));
    expect(screen.getByRole("button", { name: "Pause my list" }).hasAttribute("disabled")).toBe(true);
    expect(screen.getByRole("button", { name: "Pause all" }).hasAttribute("disabled")).toBe(false);
  });

  it("disables every button while busy", () => {
    renderCard(uiState({ mode: "allPaused" }), true);
    for (const button of screen.getAllByRole("button")) {
      expect(button.hasAttribute("disabled")).toBe(true);
    }
  });

  it("requests the mode of the clicked button", () => {
    const onSetMode = renderCard();
    screen.getByRole("button", { name: "Pause my list" }).click();
    screen.getByRole("button", { name: "Pause all" }).click();
    expect(onSetMode.mock.calls).toEqual([["customPaused"], ["allPaused"]]);
  });
});
