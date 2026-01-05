import { describe, it, expect } from "vitest";
import { AxiosError, AxiosHeaders } from "axios";
import { getErrorMessage, isUnauthorized } from "../api/errors";

describe("getErrorMessage", () => {
  it("returns message from API response", () => {
    const err = new AxiosError(
      "Request failed",
      "ERR_BAD_REQUEST",
      undefined,
      undefined,
      {
        status: 400,
        statusText: "Bad Request",
        headers: {},
        config: { headers: new AxiosHeaders() },
        data: { message: "Invalid input" },
      },
    );
    expect(getErrorMessage(err)).toBe("Invalid input");
  });

  it("returns error field from API response when message is missing", () => {
    const err = new AxiosError(
      "Request failed",
      "ERR_BAD_REQUEST",
      undefined,
      undefined,
      {
        status: 400,
        statusText: "Bad Request",
        headers: {},
        config: { headers: new AxiosHeaders() },
        data: { error: "Validation failed" },
      },
    );
    expect(getErrorMessage(err)).toBe("Validation failed");
  });

  it("returns default message when API response has no message or error", () => {
    const err = new AxiosError(
      "Request failed",
      "ERR_BAD_REQUEST",
      undefined,
      undefined,
      {
        status: 500,
        statusText: "Internal Server Error",
        headers: {},
        config: { headers: new AxiosHeaders() },
        data: {},
      },
    );
    expect(getErrorMessage(err)).toBe("Something went wrong");
  });

  it("returns network error message for network failures", () => {
    const err = new AxiosError("Network Error", "ERR_NETWORK");
    expect(getErrorMessage(err)).toBe("Unable to connect to server");
  });

  it("returns Error.message for generic errors", () => {
    const err = new Error("Something specific happened");
    expect(getErrorMessage(err)).toBe("Something specific happened");
  });

  it("returns default message for unknown error types", () => {
    expect(getErrorMessage("string error")).toBe("Something went wrong");
    expect(getErrorMessage(null)).toBe("Something went wrong");
    expect(getErrorMessage(undefined)).toBe("Something went wrong");
    expect(getErrorMessage(123)).toBe("Something went wrong");
  });
});

describe("isUnauthorized", () => {
  it("returns true for 401 responses", () => {
    const err = new AxiosError(
      "Unauthorized",
      "ERR_BAD_REQUEST",
      undefined,
      undefined,
      {
        status: 401,
        statusText: "Unauthorized",
        headers: {},
        config: { headers: new AxiosHeaders() },
        data: { message: "Invalid credentials" },
      },
    );
    expect(isUnauthorized(err)).toBe(true);
  });

  it("returns false for other status codes", () => {
    const err = new AxiosError(
      "Forbidden",
      "ERR_BAD_REQUEST",
      undefined,
      undefined,
      {
        status: 403,
        statusText: "Forbidden",
        headers: {},
        config: { headers: new AxiosHeaders() },
        data: {},
      },
    );
    expect(isUnauthorized(err)).toBe(false);
  });

  it("returns false for network errors", () => {
    const err = new AxiosError("Network Error", "ERR_NETWORK");
    expect(isUnauthorized(err)).toBe(false);
  });

  it("returns false for non-AxiosError types", () => {
    expect(isUnauthorized(new Error("Generic error"))).toBe(false);
    expect(isUnauthorized("string")).toBe(false);
    expect(isUnauthorized(null)).toBe(false);
  });
});
