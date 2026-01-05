import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/svelte";
import { AxiosError, AxiosHeaders } from "axios";
import AdminLoginForm from "../components/AdminLoginForm.svelte";

// Mock the API hook
vi.mock("../kubb/hooks/useAdminLogin", () => ({
  useAdminLogin: vi.fn(),
}));

import { useAdminLogin } from "../kubb/hooks/useAdminLogin";

describe("AdminLoginForm", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    cleanup();
  });

  it("renders login form with username and password fields", () => {
    render(AdminLoginForm);

    expect(screen.getByLabelText(/username/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /sign in/i }),
    ).toBeInTheDocument();
  });

  it("shows error when submitting with empty fields", async () => {
    render(AdminLoginForm);

    const button = screen.getByRole("button", { name: /sign in/i });
    await fireEvent.click(button);

    expect(
      screen.getByText(/please enter username and password/i),
    ).toBeInTheDocument();
  });

  it("shows error when only username is provided", async () => {
    render(AdminLoginForm);

    const usernameInput = screen.getByLabelText(/username/i);
    await fireEvent.input(usernameInput, { target: { value: "admin" } });

    const button = screen.getByRole("button", { name: /sign in/i });
    await fireEvent.click(button);

    expect(
      screen.getByText(/please enter username and password/i),
    ).toBeInTheDocument();
  });

  it("calls login API with credentials", async () => {
    const mockLogin = vi.mocked(useAdminLogin);
    mockLogin.mockResolvedValueOnce({ username: "admin" });

    const originalLocation = window.location;
    // @ts-expect-error - mocking location
    delete window.location;
    window.location = { ...originalLocation, href: "" };

    render(AdminLoginForm);

    const usernameInput = screen.getByLabelText(/username/i);
    const passwordInput = screen.getByLabelText(/password/i);

    await fireEvent.input(usernameInput, { target: { value: "  admin  " } });
    await fireEvent.input(passwordInput, { target: { value: "secret123" } });

    const button = screen.getByRole("button", { name: /sign in/i });
    await fireEvent.click(button);

    expect(mockLogin).toHaveBeenCalledWith(
      { username: "admin", password: "secret123" },
      { withCredentials: true },
    );

    window.location = originalLocation;
  });

  it("shows loading state while signing in", async () => {
    const mockLogin = vi.mocked(useAdminLogin);
    mockLogin.mockImplementationOnce(() => new Promise(() => {}));

    render(AdminLoginForm);

    const usernameInput = screen.getByLabelText(/username/i);
    const passwordInput = screen.getByLabelText(/password/i);

    await fireEvent.input(usernameInput, { target: { value: "admin" } });
    await fireEvent.input(passwordInput, { target: { value: "secret" } });

    const button = screen.getByRole("button", { name: /sign in/i });
    await fireEvent.click(button);

    expect(
      screen.getByRole("button", { name: /signing in/i }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button")).toBeDisabled();
  });

  it("shows invalid credentials error for 401 response", async () => {
    const mockLogin = vi.mocked(useAdminLogin);
    mockLogin.mockRejectedValueOnce(
      new AxiosError("Unauthorized", "ERR_BAD_REQUEST", undefined, undefined, {
        status: 401,
        statusText: "Unauthorized",
        headers: {},
        config: { headers: new AxiosHeaders() },
        data: { message: "Invalid credentials" },
      }),
    );

    render(AdminLoginForm);

    const usernameInput = screen.getByLabelText(/username/i);
    const passwordInput = screen.getByLabelText(/password/i);

    await fireEvent.input(usernameInput, { target: { value: "admin" } });
    await fireEvent.input(passwordInput, { target: { value: "wrong" } });

    const button = screen.getByRole("button", { name: /sign in/i });
    await fireEvent.click(button);

    await vi.waitFor(() => {
      expect(
        screen.getByText(/invalid username or password/i),
      ).toBeInTheDocument();
    });
  });

  it("shows API error message for non-401 errors", async () => {
    const mockLogin = vi.mocked(useAdminLogin);
    mockLogin.mockRejectedValueOnce(
      new AxiosError("Server Error", "ERR_BAD_REQUEST", undefined, undefined, {
        status: 500,
        statusText: "Internal Server Error",
        headers: {},
        config: { headers: new AxiosHeaders() },
        data: { message: "Database connection failed" },
      }),
    );

    render(AdminLoginForm);

    const usernameInput = screen.getByLabelText(/username/i);
    const passwordInput = screen.getByLabelText(/password/i);

    await fireEvent.input(usernameInput, { target: { value: "admin" } });
    await fireEvent.input(passwordInput, { target: { value: "secret" } });

    const button = screen.getByRole("button", { name: /sign in/i });
    await fireEvent.click(button);

    await vi.waitFor(() => {
      expect(
        screen.getByText(/database connection failed/i),
      ).toBeInTheDocument();
    });
  });
});
