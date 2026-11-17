import "@testing-library/jest-dom/vitest";
import { afterEach } from "vitest";
import { cleanup } from "@testing-library/react";

// Vite's `define` is not applied to Vitest's SSR transform, so provide the
// global the About page reads, exactly as the production bundle would have it.
(globalThis as Record<string, unknown>).__APP_VERSION__ = "9.9.9-test";

afterEach(() => {
  cleanup();
  localStorage.clear();
});
