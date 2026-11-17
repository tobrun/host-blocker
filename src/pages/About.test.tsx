import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import About from "./About";

describe("About", () => {
  it("shows the injected version", () => {
    render(<About />);
    expect(screen.getByText("Version")).toBeTruthy();
    expect(screen.getByText("9.9.9-test")).toBeTruthy();
  });

  it("links to the repository", () => {
    render(<About />);
    const link = screen.getByRole("link", { name: "github.com/tobrun/host-blocker" });
    expect(link.getAttribute("href")).toBe("https://github.com/tobrun/host-blocker");
    expect(link.parentElement?.textContent).toBe("Repository github.com/tobrun/host-blocker");
    expect(link.textContent).toBe("github.com/tobrun/host-blocker");
  });

  it("falls back to \"dev\" when the version global is not a string", () => {
    const previous = (globalThis as Record<string, unknown>).__APP_VERSION__;
    (globalThis as Record<string, unknown>).__APP_VERSION__ = undefined;
    try {
      render(<About />);
      expect(screen.getByText("dev")).toBeTruthy();
    } finally {
      (globalThis as Record<string, unknown>).__APP_VERSION__ = previous;
    }
  });

  it("warns about DNS-over-HTTPS bypasses", () => {
    render(<About />);
    expect(screen.getByText(/DNS-over-HTTPS/)).toBeTruthy();
  });
});
