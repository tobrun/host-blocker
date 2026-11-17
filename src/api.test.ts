import { describe, expect, it, vi } from "vitest";
import * as api from "./api";
import { ApiError } from "./api";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
const mockedInvoke = vi.mocked(invoke);

describe("api wrappers", () => {
  it("invoke arguments", async () => {
    mockedInvoke.mockResolvedValue({});
    await api.getState();
    await api.addSite("x.com");
    await api.removeSite("y.com");
    await api.importBaseline(["porn"]);
    await api.updateBaseline();
    await api.removeBaseline();
    await api.setMode("customPaused");
    await api.removeAllEntries();
    expect(mockedInvoke.mock.calls).toEqual([
      ["get_state"],
      ["add_site", { input: "x.com" }],
      ["remove_site", { input: "y.com" }],
      ["import_baseline", { extensions: ["porn"] }],
      ["update_baseline"],
      ["remove_baseline"],
      ["set_mode", { mode: "customPaused" }],
      ["remove_all_entries"],
    ]);
  });

  it("maps a serialised AppError into an ApiError", async () => {
    mockedInvoke.mockRejectedValue({ kind: "invalidHostname", message: "nope" });
    const err = await api.addSite("bad").catch((e) => e);
    expect(err).toBeInstanceOf(ApiError);
    expect(err.kind).toBe("invalidHostname");
    expect(err.message).toBe("nope");
  });

  it("maps a payload without a kind to kind 'error'", async () => {
    mockedInvoke.mockRejectedValue({ message: "boom" });
    const err = await api.getState().catch((e) => e);
    expect(err).toBeInstanceOf(ApiError);
    expect(err.kind).toBe("error");
    expect(err.message).toBe("boom");
  });

  it("maps a payload without a message to its string form", async () => {
    mockedInvoke.mockRejectedValue({ kind: "data", message: undefined });
    const err = await api.getState().catch((e) => e);
    expect(err.kind).toBe("data");
    expect(err.message).toBe("[object Object]");
  });

  it("maps a non-object rejection", async () => {
    mockedInvoke.mockRejectedValue("plain failure");
    const err = await api.getState().catch((e) => e);
    expect(err).toBeInstanceOf(ApiError);
    expect(err.kind).toBe("error");
    expect(err.message).toBe("plain failure");
  });

  it("maps a numeric rejection", async () => {
    mockedInvoke.mockRejectedValue(42);
    const err = await api.getState().catch((e) => e);
    expect(err.kind).toBe("error");
    expect(err.message).toBe("42");
  });
});
