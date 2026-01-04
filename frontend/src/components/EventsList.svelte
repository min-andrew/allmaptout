<script lang="ts">
  import { onMount } from "svelte";
  import { useListEvents } from "../kubb/hooks/useListEvents";

  let events = [];
  let loading = true;
  let error = "";

  onMount(async () => {
    try {
      const response = await useListEvents({ withCredentials: true });
      events = response.events;
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load events";
    } finally {
      loading = false;
    }
  });

  function formatDate(dateStr) {
    const date = new Date(dateStr + "T00:00:00");
    return date.toLocaleDateString("en-US", {
      weekday: "long",
      year: "numeric",
      month: "long",
      day: "numeric",
    });
  }

  function formatTime(timeStr) {
    const [hours, minutes] = timeStr.split(":");
    const h = parseInt(hours);
    const m = parseInt(minutes);
    const period = h >= 12 ? "PM" : "AM";
    const hour12 = h % 12 || 12;
    return `${hour12}:${m.toString().padStart(2, "0")} ${period}`;
  }
</script>

{#if loading}
  <div class="py-8 text-center">
    <p class="text-gray-500">Loading events...</p>
  </div>
{:else if error}
  <div class="rounded-lg bg-red-50 p-4 text-red-600">{error}</div>
{:else if events.length === 0}
  <div class="py-8 text-center">
    <p class="text-gray-500">No events scheduled yet.</p>
  </div>
{:else}
  <div class="space-y-6">
    {#each events as event (event.id)}
      <section class="rounded-lg bg-white p-6 shadow">
        <h2 class="mb-2 text-xl font-semibold capitalize text-gray-900">
          {event.name}
        </h2>
        <div class="space-y-2 text-gray-600">
          <p>
            <span class="font-medium">Date:</span>
            {formatDate(event.event_date)}
          </p>
          <p>
            <span class="font-medium">Time:</span>
            {formatTime(event.event_time)}
          </p>
          <p>
            <span class="font-medium">Location:</span>
            {event.location_name}
          </p>
          <p class="text-sm">{event.location_address}</p>
          {#if event.description}
            <p class="mt-4">{event.description}</p>
          {/if}
        </div>
      </section>
    {/each}
  </div>
{/if}
