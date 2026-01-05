import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import EventsList from "../components/EventsList.svelte";

// Mock the API hook
vi.mock("../kubb/hooks/useListEvents", () => ({
  useListEvents: vi.fn(),
}));

import { useListEvents } from "../kubb/hooks/useListEvents";

describe("EventsList", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    cleanup();
  });

  it("shows loading state initially", () => {
    const mockListEvents = vi.mocked(useListEvents);
    mockListEvents.mockImplementationOnce(() => new Promise(() => {}));

    render(EventsList);

    expect(screen.getByText(/loading events/i)).toBeInTheDocument();
  });

  it("renders events list when data is loaded", async () => {
    const mockListEvents = vi.mocked(useListEvents);
    mockListEvents.mockResolvedValueOnce({
      events: [
        {
          id: "1",
          name: "Wedding Ceremony",
          event_type: "ceremony",
          event_date: "2025-06-15",
          event_time: "14:00",
          location_name: "Beautiful Church",
          location_address: "123 Main St, City",
          description: "Join us for the ceremony",
        },
        {
          id: "2",
          name: "Reception",
          event_type: "reception",
          event_date: "2025-06-15",
          event_time: "18:00",
          location_name: "Grand Ballroom",
          location_address: "456 Party Ave, City",
          description: null,
        },
      ],
    });

    render(EventsList);

    await waitFor(() => {
      expect(screen.getByText("Wedding Ceremony")).toBeInTheDocument();
    });

    expect(screen.getByText("Reception")).toBeInTheDocument();
    expect(screen.getByText("Beautiful Church")).toBeInTheDocument();
    expect(screen.getByText("Grand Ballroom")).toBeInTheDocument();
    expect(screen.getByText("Join us for the ceremony")).toBeInTheDocument();
  });

  it("shows empty state when no events", async () => {
    const mockListEvents = vi.mocked(useListEvents);
    mockListEvents.mockResolvedValueOnce({ events: [] });

    render(EventsList);

    await waitFor(() => {
      expect(screen.getByText(/no events scheduled/i)).toBeInTheDocument();
    });
  });

  it("shows error message when API fails", async () => {
    const mockListEvents = vi.mocked(useListEvents);
    mockListEvents.mockRejectedValueOnce(new Error("Network error"));

    render(EventsList);

    await waitFor(() => {
      expect(screen.getByText(/network error/i)).toBeInTheDocument();
    });
  });

  it("formats date correctly", async () => {
    const mockListEvents = vi.mocked(useListEvents);
    mockListEvents.mockResolvedValueOnce({
      events: [
        {
          id: "1",
          name: "Test Event",
          event_type: "ceremony",
          event_date: "2025-06-15",
          event_time: "14:00",
          location_name: "Venue",
          location_address: "Address",
          description: null,
        },
      ],
    });

    render(EventsList);

    await waitFor(() => {
      // Should show "Sunday, June 15, 2025"
      expect(screen.getByText(/june 15, 2025/i)).toBeInTheDocument();
    });
  });

  it("formats time correctly (PM)", async () => {
    const mockListEvents = vi.mocked(useListEvents);
    mockListEvents.mockResolvedValueOnce({
      events: [
        {
          id: "1",
          name: "Test Event",
          event_type: "ceremony",
          event_date: "2025-06-15",
          event_time: "14:30",
          location_name: "Venue",
          location_address: "Address",
          description: null,
        },
      ],
    });

    render(EventsList);

    await waitFor(() => {
      expect(screen.getByText(/2:30 PM/i)).toBeInTheDocument();
    });
  });

  it("formats time correctly (AM)", async () => {
    const mockListEvents = vi.mocked(useListEvents);
    mockListEvents.mockResolvedValueOnce({
      events: [
        {
          id: "1",
          name: "Brunch",
          event_type: "brunch",
          event_date: "2025-06-16",
          event_time: "10:00",
          location_name: "Cafe",
          location_address: "Address",
          description: null,
        },
      ],
    });

    render(EventsList);

    await waitFor(() => {
      expect(screen.getByText(/10:00 AM/i)).toBeInTheDocument();
    });
  });
});
