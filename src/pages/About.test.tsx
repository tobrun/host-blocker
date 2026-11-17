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
  });

  it("warns about DNS-over-HTTPS bypasses", () => {
    render(<About />);
    expect(screen.getByText(/DNS-over-HTTPS/)).toBeTruthy();
  });
});
