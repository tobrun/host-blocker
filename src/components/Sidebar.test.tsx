import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import Sidebar from "./Sidebar";
import type { UiState } from "../types";

const state: UiState = {
  mode: "active",
  sites: [],
  baseline: null,
  blockedCount: 3,
};

describe("Sidebar", () => {
  it("renders the brand and all nav entries", () => {
    render(<Sidebar page="lists" onNavigate={() => {}} state={state} busy={false} onSetMode={() => {}} />);
    expect(screen.getByRole("heading", { name: "HostBlocker" })).toBeTruthy();
    for (const label of ["Block Lists", "Settings", "About"]) {
      expect(screen.getByRole("button", { name: label })).toBeTruthy();
    }
  });

  it("marks the current page and moves aria-current on navigation", () => {
    const onNavigate = vi.fn();
    render(<Sidebar page="lists" onNavigate={onNavigate} state={state} busy={false} onSetMode={() => {}} />);
    expect(screen.getByRole("button", { name: "Block Lists" }).getAttribute("aria-current")).toBe("page");
    expect(screen.getByRole("button", { name: "Block Lists" }).className).toContain("active");
    expect(screen.getByRole("button", { name: "About" }).getAttribute("aria-current")).toBeNull();
    expect(screen.getByRole("button", { name: "About" }).className.trim()).toBe("nav-item");
    expect(screen.getByRole("button", { name: "Block Lists" }).className.trim()).toBe("nav-item active");

    fireEvent.click(screen.getByRole("button", { name: "About" }));
    expect(onNavigate).toHaveBeenCalledWith("about");
  });

  it("shows a placeholder instead of the status card while loading", () => {
    render(<Sidebar page="lists" onNavigate={() => {}} state={null} busy={false} onSetMode={() => {}} />);
    expect(screen.getByText(/Loading status/)).toBeTruthy();
    expect(screen.queryByLabelText("Blocking status")).toBeNull();
  });

  it("embeds the status card and forwards mode changes", () => {
    const onSetMode = vi.fn();
    render(<Sidebar page="about" onNavigate={() => {}} state={state} busy={false} onSetMode={onSetMode} />);
    expect(screen.getByLabelText("Blocking status")).toBeTruthy();
    screen.getByRole("button", { name: "Pause all" }).click();
    expect(onSetMode).toHaveBeenCalledWith("allPaused");
  });
});
