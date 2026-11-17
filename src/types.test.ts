import { describe, expect, it } from "vitest";
import { SOURCES, baselineLabel, sourceInfo } from "./types";
import type { BaselineView } from "./types";

function baseline(overrides: Partial<BaselineView> & { source: BaselineView["source"] }): BaselineView {
  return { variant: null, extensions: [], hostCount: 1, fetchedAt: "", ...overrides };
}

describe("sourceInfo", () => {
  it("knows every offered list", () => {
    const ids = SOURCES.map((s) => s.id);
    expect(new Set(ids).size).toBe(ids.length);
    for (const source of SOURCES) {
      expect(sourceInfo(source.id)).toBe(source);
    }
  });
});

describe("baselineLabel", () => {
  it("names the active source by its wire id", () => {
    expect(baselineLabel(baseline({ source: "adaway" }))).toBe("adaway");
    expect(baselineLabel(baseline({ source: "oisd-small" }))).toBe("oisd-small");
    expect(baselineLabel(baseline({ source: "blocklistproject-phishing" }))).toBe(
      "blocklistproject-phishing",
    );
  });

  it("appends the variant slug when one is active", () => {
    expect(
      baselineLabel(baseline({ source: "stevenblack", variant: "fakenews-gambling" })),
    ).toBe("stevenblack fakenews-gambling");
  });
});
