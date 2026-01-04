import { defineMiddleware } from "astro:middleware";

const API_URL =
  import.meta.env.PUBLIC_API_URL ||
  (import.meta.env.DEV ? "http://localhost:3001" : "http://backend:3001");

type SessionResponse = {
  session_type: "guest" | "admin_pending" | "admin";
  guest_id: string | null;
  guest_name: string | null;
  admin_id: string | null;
  admin_username: string | null;
};

async function getSession(
  cookie: string | null,
): Promise<SessionResponse | null> {
  if (!cookie) return null;

  try {
    const response = await fetch(`${API_URL}/auth/session`, {
      headers: { Cookie: cookie },
    });
    if (!response.ok) return null;
    return response.json();
  } catch {
    return null;
  }
}

export const onRequest = defineMiddleware(async (context, next) => {
  const { pathname } = context.url;
  const cookie = context.request.headers.get("cookie");

  // Health check - no session needed
  if (pathname === "/health") {
    return next();
  }

  const session = await getSession(cookie);

  // Landing page - redirect if already logged in
  if (pathname === "/") {
    if (session) {
      if (session.session_type === "guest") {
        return context.redirect("/events");
      } else if (session.session_type === "admin_pending") {
        return context.redirect("/admin/login");
      } else if (session.session_type === "admin") {
        return context.redirect("/admin");
      }
    }
    return next();
  }

  // Admin login page - requires admin_pending session
  if (pathname === "/admin/login") {
    if (!session || session.session_type !== "admin_pending") {
      return context.redirect("/");
    }
    return next();
  }

  // Admin routes - require full admin session
  if (pathname.startsWith("/admin")) {
    if (!session || session.session_type !== "admin") {
      return context.redirect("/");
    }
    // Make session available to the page
    context.locals.session = session;
    return next();
  }

  // Guest routes (events, rsvp, etc.) - require guest session
  if (
    pathname.startsWith("/events") ||
    pathname.startsWith("/rsvp") ||
    pathname.startsWith("/faq") ||
    pathname.startsWith("/gallery")
  ) {
    if (!session || session.session_type !== "guest") {
      return context.redirect("/");
    }
    // Make session available to the page
    context.locals.session = session;
    return next();
  }

  // All other routes pass through
  return next();
});
