import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/svelte";
import LogoutButton from "../components/LogoutButton.svelte";

// Mock the API hook
vi.mock("../kubb/hooks/useLogout", () => ({
  useLogout: vi.fn(),
}));

import { useLogout } from "../kubb/hooks/useLogout";

describe("LogoutButton", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    cleanup();
  });

  it("renders logout button", () => {
    render(LogoutButton);
    expect(screen.getByRole("button", { name: /logout/i })).toBeInTheDocument();
  });

  it("calls logout API and redirects on click", async () => {
    const mockLogout = vi.mocked(useLogout);
    mockLogout.mockResolvedValueOnce(undefined);

    // Mock window.location
    const originalLocation = window.location;
    // @ts-expect-error - mocking location
    delete window.location;
    window.location = { ...originalLocation, href: "" };

    render(LogoutButton);

    const button = screen.getByRole("button", { name: /logout/i });
    await fireEvent.click(button);

    expect(mockLogout).toHaveBeenCalledWith({ withCredentials: true });

    // Wait for redirect
    await vi.waitFor(() => {
      expect(window.location.href).toBe("/");
    });

    window.location = originalLocation;
  });

  it("shows loading state while logging out", async () => {
    const mockLogout = vi.mocked(useLogout);
    mockLogout.mockImplementationOnce(() => new Promise(() => {}));

    render(LogoutButton);

    const button = screen.getByRole("button", { name: /logout/i });
    await fireEvent.click(button);

    expect(screen.getByRole("button")).toHaveTextContent("...");
    expect(screen.getByRole("button")).toBeDisabled();
  });

  it("redirects even if logout API fails", async () => {
    const mockLogout = vi.mocked(useLogout);
    mockLogout.mockRejectedValueOnce(new Error("Network error"));

    const originalLocation = window.location;
    // @ts-expect-error - mocking location
    delete window.location;
    window.location = { ...originalLocation, href: "" };

    render(LogoutButton);

    const button = screen.getByRole("button", { name: /logout/i });
    await fireEvent.click(button);

    await vi.waitFor(() => {
      expect(window.location.href).toBe("/");
    });

    window.location = originalLocation;
  });
});
