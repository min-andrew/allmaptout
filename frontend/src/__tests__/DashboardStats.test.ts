import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import DashboardStats from "../components/DashboardStats.svelte";

// Mock the API hook
vi.mock("../kubb/hooks/useGetDashboardStats", () => ({
  useGetDashboardStats: vi.fn(),
}));

import { useGetDashboardStats } from "../kubb/hooks/useGetDashboardStats";

describe("DashboardStats", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    cleanup();
  });

  it("shows loading state initially", () => {
    const mockGetStats = vi.mocked(useGetDashboardStats);
    mockGetStats.mockImplementationOnce(() => new Promise(() => {}));

    render(DashboardStats);

    expect(screen.getByText(/loading dashboard/i)).toBeInTheDocument();
  });

  it("displays all stat cards when data loads", async () => {
    const mockGetStats = vi.mocked(useGetDashboardStats);
    mockGetStats.mockResolvedValueOnce({
      total_guests: 50,
      total_expected_attendees: 120,
      rsvp_count: 30,
      pending_rsvps: 20,
      attending_count: 85,
      not_attending_count: 15,
      recent_rsvps: [],
    });

    render(DashboardStats);

    await waitFor(() => {
      expect(screen.getByText("50")).toBeInTheDocument();
    });

    expect(screen.getByText("120 expected attendees")).toBeInTheDocument();
    expect(screen.getByText("30")).toBeInTheDocument();
    expect(screen.getByText("20 pending")).toBeInTheDocument();
    expect(screen.getByText("85")).toBeInTheDocument();
    expect(screen.getByText("15")).toBeInTheDocument();
  });

  it("shows RSVP progress bar", async () => {
    const mockGetStats = vi.mocked(useGetDashboardStats);
    mockGetStats.mockResolvedValueOnce({
      total_guests: 100,
      total_expected_attendees: 200,
      rsvp_count: 50,
      pending_rsvps: 50,
      attending_count: 100,
      not_attending_count: 25,
      recent_rsvps: [],
    });

    render(DashboardStats);

    await waitFor(() => {
      expect(screen.getByText("50 of 100 responded")).toBeInTheDocument();
    });
  });

  it("displays recent RSVPs", async () => {
    const mockGetStats = vi.mocked(useGetDashboardStats);
    mockGetStats.mockResolvedValueOnce({
      total_guests: 10,
      total_expected_attendees: 25,
      rsvp_count: 5,
      pending_rsvps: 5,
      attending_count: 15,
      not_attending_count: 3,
      recent_rsvps: [
        {
          guest_name: "John Smith",
          responded_at: "2025-06-01T10:30:00Z",
          attending_count: 2,
          not_attending_count: 0,
        },
        {
          guest_name: "Jane Doe",
          responded_at: "2025-05-30T14:00:00Z",
          attending_count: 1,
          not_attending_count: 1,
        },
      ],
    });

    render(DashboardStats);

    await waitFor(() => {
      expect(screen.getByText("John Smith")).toBeInTheDocument();
    });

    expect(screen.getByText("Jane Doe")).toBeInTheDocument();
    expect(screen.getByText("2 attending")).toBeInTheDocument();
    expect(screen.getByText("1 attending")).toBeInTheDocument();
    expect(screen.getByText("1 declined")).toBeInTheDocument();
  });

  it("shows empty state for no recent RSVPs", async () => {
    const mockGetStats = vi.mocked(useGetDashboardStats);
    mockGetStats.mockResolvedValueOnce({
      total_guests: 10,
      total_expected_attendees: 20,
      rsvp_count: 0,
      pending_rsvps: 10,
      attending_count: 0,
      not_attending_count: 0,
      recent_rsvps: [],
    });

    render(DashboardStats);

    await waitFor(() => {
      expect(screen.getByText(/no rsvps yet/i)).toBeInTheDocument();
    });
  });

  it("shows error message when API fails", async () => {
    const mockGetStats = vi.mocked(useGetDashboardStats);
    mockGetStats.mockRejectedValueOnce(new Error("Failed to fetch"));

    render(DashboardStats);

    await waitFor(() => {
      expect(screen.getByText(/failed to fetch/i)).toBeInTheDocument();
    });
  });
});
