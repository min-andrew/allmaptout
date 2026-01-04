import { client } from "./client";

export type ValidateCodeResponse = {
  session_type: "guest" | "admin_pending";
  guest_name: string | null;
};

export type AdminLoginResponse = {
  username: string;
};

export type SessionResponse = {
  session_type: "guest" | "admin_pending" | "admin";
  guest_id: string | null;
  guest_name: string | null;
  admin_id: string | null;
  admin_username: string | null;
};

export async function validateCode(
  code: string,
): Promise<ValidateCodeResponse> {
  return client<ValidateCodeResponse>({
    method: "POST",
    url: "/auth/code",
    data: { code },
  });
}

export async function adminLogin(
  username: string,
  password: string,
): Promise<AdminLoginResponse> {
  return client<AdminLoginResponse>({
    method: "POST",
    url: "/auth/admin/login",
    data: { username, password },
  });
}

export async function logout(): Promise<void> {
  await client<void>({
    method: "POST",
    url: "/auth/logout",
  });
}

export async function getSession(): Promise<SessionResponse> {
  return client<SessionResponse>({
    method: "GET",
    url: "/auth/session",
  });
}
