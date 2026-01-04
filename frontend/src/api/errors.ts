import { AxiosError } from "axios";

interface ApiErrorData {
  message?: string;
  error?: string;
}

export function getErrorMessage(err: unknown): string {
  if (err instanceof AxiosError) {
    if (err.response) {
      const data = err.response.data as ApiErrorData;
      return data?.message || data?.error || "Something went wrong";
    }
    if (err.code === "ERR_NETWORK") {
      return "Unable to connect to server";
    }
  }
  if (err instanceof Error) {
    return err.message;
  }
  return "Something went wrong";
}

export function isUnauthorized(err: unknown): boolean {
  return err instanceof AxiosError && err.response?.status === 401;
}
