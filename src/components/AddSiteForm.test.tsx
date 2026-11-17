import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import AddSiteForm from "./AddSiteForm";

describe("AddSiteForm", () => {
  it("submits the trimmed input and clears the field on success", async () => {
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    render(<AddSiteForm busy={false} onSubmit={onSubmit} />);

    const input = screen.getByLabelText("Website URL");
    fireEvent.change(input, { target: { value: "  https://www.reddit.com/r/all  " } });
    fireEvent.click(screen.getByRole("button", { name: "Add" }));

    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    expect(onSubmit).toHaveBeenCalledWith("https://www.reddit.com/r/all");
    expect(input).toHaveProperty("value", "");
  });

  it("keeps the input and shows the error when submitting fails", async () => {
    const onSubmit = vi.fn().mockRejectedValue(new Error(`"not a host" is not a valid hostname`));
    render(<AddSiteForm busy={false} onSubmit={onSubmit} />);

    const input = screen.getByLabelText("Website URL");
    fireEvent.change(input, { target: { value: "not a host" } });
    fireEvent.click(screen.getByRole("button", { name: "Add" }));

    const error = await screen.findByRole("alert");
    expect(error.textContent).toContain("not a valid hostname");
    expect(input).toHaveProperty("value", "not a host");
  });

  it("disables submit while empty or busy", () => {
    render(<AddSiteForm busy={false} onSubmit={vi.fn()} />);
    const submit = screen.getByRole("button", { name: "Add" });
    expect(submit.hasAttribute("disabled")).toBe(true);

    fireEvent.change(screen.getByLabelText("Website URL"), { target: { value: "example.com" } });
    expect(submit.hasAttribute("disabled")).toBe(false);
  });

  it("does not submit twice while a submission is pending", async () => {
    let release: () => void = () => {};
    const onSubmit = vi.fn().mockImplementation(
      () => new Promise<void>((resolve) => (release = resolve)),
    );
    render(<AddSiteForm busy={false} onSubmit={onSubmit} />);

    fireEvent.change(screen.getByLabelText("Website URL"), { target: { value: "example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Add" }));
    fireEvent.click(screen.getByRole("button", { name: "Applying…" }));
    expect(onSubmit).toHaveBeenCalledTimes(1);

    release();
    await waitFor(() => expect(screen.getByRole("button", { name: "Add" })).toBeTruthy());
  });
});
