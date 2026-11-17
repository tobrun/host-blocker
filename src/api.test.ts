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

  it("only enters the payload branch for objects that actually carry a message", async () => {
    // No `message` key at all: the `"message" in err` guard must send this
    // down the plain-error path even though a kind is present.
    mockedInvoke.mockRejectedValue({ kind: "data" });
    const keyMissing = await api.getState().catch((e) => e);
    expect(keyMissing).toBeInstanceOf(ApiError);
    expect(keyMissing.kind).toBe("error");

    // null must not deref into the payload branch ("message" in null throws).
    mockedInvoke.mockRejectedValue(null);
    const nulled = await api.getState().catch((e) => e);
    expect(nulled.kind).toBe("error");
    expect(nulled.message).toBe("null");

    // A non-object (function) that carries a `message` property must still
    // take the plain path, because typeof is not "object".
    const fn = Object.assign(function withMessage() {}, { message: "sneaky" });
    mockedInvoke.mockRejectedValue(fn);
    const callable = await api.getState().catch((e) => e);
    expect(callable.kind).toBe("error");
    expect(callable.message).toBe(String(fn));
  });
});
