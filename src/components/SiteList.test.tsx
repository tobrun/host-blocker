import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import SiteList from "./SiteList";

describe("SiteList", () => {
  it("renders one row per site", () => {
    render(<SiteList sites={["youtube.com", "twitter.com"]} busy={false} onRemove={() => {}} />);
    expect(screen.getByText("youtube.com")).toBeTruthy();
    expect(screen.getByText("twitter.com")).toBeTruthy();
    expect(screen.getAllByText("4 hosts-file entries")).toHaveLength(2);
  });

  it("shows an empty hint when there are no sites", () => {
    render(<SiteList sites={[]} busy={false} onRemove={() => {}} />);
    expect(screen.getByText(/No sites yet/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Remove" })).toBeNull();
  });

  it("removes the matching site only", () => {
    const onRemove = vi.fn();
    render(<SiteList sites={["a.com", "b.com"]} busy={false} onRemove={onRemove} />);
    screen.getAllByRole("button", { name: "Remove" })[1].click();
    expect(onRemove).toHaveBeenCalledTimes(1);
    expect(onRemove).toHaveBeenCalledWith("b.com");
  });

  it("disables remove buttons while busy", () => {
    render(<SiteList sites={["a.com"]} busy onRemove={() => {}} />);
    expect(screen.getByRole("button", { name: "Remove" }).hasAttribute("disabled")).toBe(true);
  });
});
