import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/svelte";
import CodeInput from "../components/CodeInput.svelte";

// Mock the API hook
vi.mock("../kubb/hooks/useValidateCode", () => ({
  useValidateCode: vi.fn(),
}));

import { useValidateCode } from "../kubb/hooks/useValidateCode";

describe("CodeInput", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    cleanup();
  });

  it("renders the form with input and button", () => {
    render(CodeInput);

    expect(
      screen.getByLabelText(/enter your invite code/i),
    ).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/SMITH2024/i)).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /continue/i }),
    ).toBeInTheDocument();
  });

  it("shows error when submitting empty code", async () => {
    render(CodeInput);

    const button = screen.getByRole("button", { name: /continue/i });
    await fireEvent.click(button);

    expect(screen.getByText(/please enter a code/i)).toBeInTheDocument();
  });

  it("calls API with trimmed code on submit", async () => {
    const mockValidate = vi.mocked(useValidateCode);
    mockValidate.mockResolvedValueOnce({
      session_type: "guest",
      guest_name: "Test Guest",
    });

    // Mock window.location
    const locationSpy = vi.spyOn(window, "location", "get").mockReturnValue({
      ...window.location,
      href: "",
    } as Location);

    render(CodeInput);

    const input = screen.getByPlaceholderText(/SMITH2024/i);
    await fireEvent.input(input, { target: { value: "  ABC123  " } });

    const button = screen.getByRole("button", { name: /continue/i });
    await fireEvent.click(button);

    expect(mockValidate).toHaveBeenCalledWith(
      { code: "ABC123" },
      { withCredentials: true },
    );

    locationSpy.mockRestore();
  });

  it("shows loading state while submitting", async () => {
    const mockValidate = vi.mocked(useValidateCode);
    // Never resolve to keep loading state
    mockValidate.mockImplementationOnce(() => new Promise(() => {}));

    render(CodeInput);

    const input = screen.getByPlaceholderText(/SMITH2024/i);
    await fireEvent.input(input, { target: { value: "ABC123" } });

    const button = screen.getByRole("button", { name: /continue/i });
    await fireEvent.click(button);

    expect(
      screen.getByRole("button", { name: /checking/i }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button")).toBeDisabled();
  });

  it("displays API error message on failure", async () => {
    const mockValidate = vi.mocked(useValidateCode);
    mockValidate.mockRejectedValueOnce(new Error("Invalid code"));

    render(CodeInput);

    const input = screen.getByPlaceholderText(/SMITH2024/i);
    await fireEvent.input(input, { target: { value: "BADCODE" } });

    const button = screen.getByRole("button", { name: /continue/i });
    await fireEvent.click(button);

    // Wait for error to appear
    await vi.waitFor(() => {
      expect(screen.getByText(/invalid code/i)).toBeInTheDocument();
    });
  });
});
