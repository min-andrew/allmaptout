/// <reference types="astro/client" />

type SessionResponse = {
  session_type: "guest" | "admin_pending" | "admin";
  guest_id: string | null;
  guest_name: string | null;
  admin_id: string | null;
  admin_username: string | null;
};

declare namespace App {
  interface Locals {
    session?: SessionResponse;
  }
}
