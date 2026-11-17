import { render, screen, waitFor, fireEvent, within } from "@testing-library/react";
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
    source: "stevenblack",
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

// The baseline card also has a "Remove" button, so reach into the first
// <li> to click a site's own Remove.
function firstSiteRemoveButton() {
  return within(screen.getAllByRole("listitem")[0]).getByRole("button", {
    name: "Remove",
  });
}

describe("App", () => {
  it("loads the state and renders the shell", async () => {
    mockedInvoke.mockImplementation(async () => mockedState());
    render(<App />);

    expect(await screen.findByText("HostBlocker")).toBeTruthy();
    expect(mockedInvoke).toHaveBeenCalledWith("get_state");
    expect(await screen.findByText("59008 hosts blocked")).toBeTruthy();
    // The empty useEffect deps must not re-trigger the initial fetch on
    // re-render (kills an ArrayDeclaration mutant on the deps).
    expect(mockedInvoke.mock.calls.filter((c) => c[0] === "get_state")).toHaveLength(1);
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
    // Navigation re-renders App; the initial fetch must not re-run (kills an
    // ArrayDeclaration mutant on the useEffect deps).
    expect(mockedInvoke.mock.calls.filter((c) => c[0] === "get_state")).toHaveLength(1);
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

    firstSiteRemoveButton().click();
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
    firstSiteRemoveButton().click();
    await screen.findByRole("alert");

    failing = false;
    const pauseAll = screen.getByRole("button", { name: "Pause all" });
    expect(pauseAll.hasAttribute("disabled")).toBe(false);
    fireEvent.click(pauseAll);
    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("set_mode", { mode: "allPaused" }),
    );
    // A successful run must clear the previous notice.
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("marks the app busy while a change is in flight", async () => {
    let release!: (s: UiState) => void;
    const pending = new Promise<UiState>((res) => {
      release = res;
    });
    let second = false;
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_state") return mockedState();
      second = true;
      return pending;
    });
    render(<App />);
    await screen.findByText("HostBlocker");

    firstSiteRemoveButton().click();
    await waitFor(() => expect(second).toBe(true));
    expect(screen.getByRole("button", { name: "Pause all" }).hasAttribute("disabled")).toBe(true);

    release(mockedState());
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Pause all" }).hasAttribute("disabled")).toBe(false),
    );
  });

  it("wires baseline import, update and remove through to the backend", async () => {
    const imported = mockedState();
    imported.baseline = {
      source: "stevenblack",
      variant: "fakenews-gambling-porn",
      extensions: ["fakenews"],
      hostCount: 60000,
      fetchedAt: "2026-11-15T10:00:00Z",
    };
    let first = true;
    mockedInvoke.mockImplementation(async () => {
      if (first) {
        first = false;
        const empty = mockedState();
        empty.baseline = null;
        return empty;
      }
      return imported;
    });
    render(<App />);
    await screen.findByText("HostBlocker");

    fireEvent.click(screen.getByRole("button", { name: "Import StevenBlack Hosts" }));
    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("import_baseline", {
        source: "stevenblack",
        extensions: [],
      }),
    );

    // With a baseline imported, the card now offers Update / Remove.
    fireEvent.click(await screen.findByRole("button", { name: "Update" }));
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("update_baseline"));

    // Baseline Remove is the first of the two; site rows follow the card.
    fireEvent.click(screen.getAllByRole("button", { name: "Remove" })[0]);
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("remove_baseline"));
  });

  it("removes a single site through the row button", async () => {
    mockedInvoke.mockImplementation(async () => mockedState());
    render(<App />);
    await screen.findByText("HostBlocker");

    const siteRows = screen.getAllByRole("listitem");
    fireEvent.click(within(siteRows[0]).getByRole("button", { name: "Remove" }));
    await waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("remove_site", { input: "youtube.com" }),
    );
  });

  it("wires the settings page actions through to the backend", async () => {
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);
    mockedInvoke.mockImplementation(async () => mockedState());
    render(<App />);
    await screen.findByText("HostBlocker");
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));

    fireEvent.click(screen.getByRole("button", { name: "Update now" }));
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("update_baseline"));

    fireEvent.click(screen.getByRole("button", { name: "Remove all entries" }));
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("remove_all_entries"));
    confirmSpy.mockRestore();
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

    firstSiteRemoveButton().click();
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("administrator prompt was cancelled");
    fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
    expect(screen.queryByRole("alert")).toBeNull();
  });
});
