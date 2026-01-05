<script lang="ts">
  import { onMount } from "svelte";
  import { useGetDashboardStats } from "../kubb/hooks/useGetDashboardStats";
  import { getErrorMessage } from "../api/errors";

  let stats = null;
  let loading = true;
  let error = "";

  onMount(async () => {
    try {
      stats = await useGetDashboardStats({ withCredentials: true });
    } catch (err) {
      error = getErrorMessage(err);
    } finally {
      loading = false;
    }
  });

  function formatDate(dateStr) {
    const date = new Date(dateStr);
    return date.toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    });
  }
</script>

{#if loading}
  <div class="py-8 text-center">
    <p class="text-gray-500">Loading dashboard...</p>
  </div>
{:else if error}
  <div class="rounded-lg bg-red-50 p-4 text-red-600">{error}</div>
{:else if stats}
  <div class="space-y-6">
    <!-- Summary Cards -->
    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
      <div class="rounded-lg bg-white p-6 shadow">
        <p class="text-sm font-medium text-gray-500">Total Guests</p>
        <p class="mt-1 text-3xl font-bold text-gray-900">
          {stats.total_guests}
        </p>
        <p class="mt-1 text-sm text-gray-500">
          {stats.total_expected_attendees} expected attendees
        </p>
      </div>

      <div class="rounded-lg bg-white p-6 shadow">
        <p class="text-sm font-medium text-gray-500">RSVPs Received</p>
        <p class="mt-1 text-3xl font-bold text-blue-600">{stats.rsvp_count}</p>
        <p class="mt-1 text-sm text-gray-500">
          {stats.pending_rsvps} pending
        </p>
      </div>

      <div class="rounded-lg bg-white p-6 shadow">
        <p class="text-sm font-medium text-gray-500">Attending</p>
        <p class="mt-1 text-3xl font-bold text-green-600">
          {stats.attending_count}
        </p>
        <p class="mt-1 text-sm text-gray-500">confirmed guests</p>
      </div>

      <div class="rounded-lg bg-white p-6 shadow">
        <p class="text-sm font-medium text-gray-500">Not Attending</p>
        <p class="mt-1 text-3xl font-bold text-red-600">
          {stats.not_attending_count}
        </p>
        <p class="mt-1 text-sm text-gray-500">declined</p>
      </div>
    </div>

    <!-- Progress Bar -->
    <div class="rounded-lg bg-white p-6 shadow">
      <div class="mb-2 flex items-center justify-between">
        <p class="text-sm font-medium text-gray-700">RSVP Progress</p>
        <p class="text-sm text-gray-500">
          {stats.rsvp_count} of {stats.total_guests} responded
        </p>
      </div>
      <div class="h-4 w-full overflow-hidden rounded-full bg-gray-200">
        {#if stats.total_guests > 0}
          <div
            class="h-full rounded-full bg-blue-600 transition-all duration-500"
            style="width: {(stats.rsvp_count / stats.total_guests) * 100}%"
          ></div>
        {/if}
      </div>
    </div>

    <!-- Recent Activity -->
    <div class="rounded-lg bg-white p-6 shadow">
      <h3 class="mb-4 text-lg font-semibold text-gray-900">Recent RSVPs</h3>
      {#if stats.recent_rsvps.length === 0}
        <p class="text-gray-500">No RSVPs yet.</p>
      {:else}
        <div class="space-y-3">
          {#each stats.recent_rsvps as rsvp, i (i)}
            <div
              class="flex items-center justify-between border-b border-gray-100 pb-3 last:border-0"
            >
              <div>
                <p class="font-medium text-gray-900">{rsvp.guest_name}</p>
                <p class="text-sm text-gray-500">
                  {formatDate(rsvp.responded_at)}
                </p>
              </div>
              <div class="text-right">
                {#if rsvp.attending_count > 0}
                  <span
                    class="inline-flex rounded-full bg-green-100 px-2 py-1 text-xs font-medium text-green-700"
                  >
                    {rsvp.attending_count} attending
                  </span>
                {/if}
                {#if rsvp.not_attending_count > 0}
                  <span
                    class="ml-1 inline-flex rounded-full bg-red-100 px-2 py-1 text-xs font-medium text-red-700"
                  >
                    {rsvp.not_attending_count} declined
                  </span>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{/if}
