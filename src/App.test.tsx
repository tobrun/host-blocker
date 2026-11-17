import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import App from "./App";
import type { UiState } from "./types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
const mockedInvoke = vi.mocked(invoke);

const initialState: UiState = {
  mode: "active",
  sites: ["youtube.com", "news.ycombinator.com"],
  baseline: {
    variant: null,
    extensions: [],
    hostCount: 59000,
    fetchedAt: "2026-11-01T10:00:00Z",
  },
  blockedCount: 59008,
};

function mockedState(): UiState {
  return structuredClone(initialState);
}

describe("App", () => {
  it("loads the state and renders the shell", async () => {
    mockedInvoke.mockImplementation(async () => mockedState());
    render(<App />);

    expect(await screen.findByText("HostBlocker")).toBeTruthy();
    expect(mockedInvoke).toHaveBeenCalledWith("get_state");
    expect(await screen.findByText("59008 hosts blocked")).toBeTruthy();
  });

  it("navigates between pages", async () => {
    mockedInvoke.mockImplementation(async () => mockedState());
    render(<App />);
    await screen.findByText("HostBlocker");

    expect(screen.getByRole("heading", { name: "Block Lists" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    expect(screen.getByRole("heading", { name: "Settings" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "About" }));
    expect(screen.getByRole("heading", { name: "About" })).toBeTruthy();
  });

  it("filters the site list through the search box, case- and space-insensitively", async () => {
    mockedInvoke.mockImplementation(async () => mockedState());
    render(<App />);
    const search = await screen.findByRole("searchbox");

    fireEvent.change(search, { target: { value: "  YouTuBe " } });
    expect(screen.getByText("youtube.com")).toBeTruthy();
    expect(screen.queryByText("news.ycombinator.com")).toBeNull();
  });

  it("sends set_mode when a mode button is clicked", async () => {
    mockedInvoke.mockImplementation(async () => mockedState());
    render(<App />);
    await screen.findByText("HostBlocker");

    fireEvent.click(screen.getByRole("button", { name: "Pause all" }));
    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("set_mode", { mode: "allPaused" }),
    );
  });

  it("surfaces a get_state failure as a notice", async () => {
    mockedInvoke.mockRejectedValue(new Error("state file corrupt"));
    render(<App />);
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("state file corrupt");
  });

  it("shows the raw message for non-cancelled failures", async () => {
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_state") return mockedState();
      throw { kind: "data", message: "cache vanished" };
    });
    render(<App />);
    await screen.findByText("HostBlocker");

    screen.getAllByRole("button", { name: "Remove" })[0].click();
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("cache vanished");
    expect(alert.textContent).not.toContain("administrator");
  });

  it("keeps the app interactive after a failed change", async () => {
    let failing = false;
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_state") return mockedState();
      if (failing) throw { kind: "cancelled", message: "declined" };
      return mockedState();
    });
    render(<App />);
    await screen.findByText("HostBlocker");

    failing = true;
    screen.getAllByRole("button", { name: "Remove" })[0].click();
    await screen.findByRole("alert");

    failing = false;
    const pauseAll = screen.getByRole("button", { name: "Pause all" });
    expect(pauseAll.hasAttribute("disabled")).toBe(false);
    fireEvent.click(pauseAll);
    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("set_mode", { mode: "allPaused" }),
    );
  });

  it("shows add-site errors inline in the form, not as a notice", async () => {
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_state") return mockedState();
      throw { kind: "invalidHostname", message: `"nope" is not a valid hostname` };
    });
    render(<App />);
    await screen.findByText("HostBlocker");

    fireEvent.change(screen.getByLabelText("Website URL"), { target: { value: "nope" } });
    fireEvent.click(screen.getByRole("button", { name: "Add" }));

    const inline = await screen.findByText(/is not a valid hostname/);
    expect(inline.closest(".form-error")).toBeTruthy();
    expect(screen.queryByRole("alert")?.classList.contains("notice")).toBeFalsy();
  });

  it("sends add_site with the URL and adopts the returned state", async () => {
    const next = mockedState();
    next.sites.push("reddit.com");
    let first = true;
    mockedInvoke.mockImplementation(async () => {
      if (first) {
        first = false;
        return mockedState();
      }
      return next;
    });
    render(<App />);
    await screen.findByText("HostBlocker");

    fireEvent.change(screen.getByLabelText("Website URL"), {
      target: { value: "https://www.reddit.com/r/all" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Add" }));

    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("add_site", {
      input: "https://www.reddit.com/r/all",
    }));
    expect(await screen.findByText("reddit.com")).toBeTruthy();
  });

  it("surfaces a cancelled admin prompt as a dismissible notice", async () => {
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_state") return mockedState();
      throw { kind: "cancelled", message: "User declined authorisation." };
    });
    render(<App />);
    await screen.findByText("HostBlocker");

    screen.getAllByRole("button", { name: "Remove" })[0].click();
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("administrator prompt was cancelled");
    fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
    expect(screen.queryByRole("alert")).toBeNull();
  });
});
