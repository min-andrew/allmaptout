<script lang="ts">
  import { onMount } from "svelte";
  import { useListAdminEvents } from "../kubb/hooks/useListAdminEvents";
  import { useCreateEvent } from "../kubb/hooks/useCreateEvent";
  import { useUpdateEvent } from "../kubb/hooks/useUpdateEvent";
  import { useDeleteEvent } from "../kubb/hooks/useDeleteEvent";
  import { getErrorMessage } from "../api/errors";

  let events = [];
  let loading = true;
  let error = "";

  // Form state
  let showForm = false;
  let editingEvent = null;
  let formData = {
    name: "",
    event_type: "ceremony",
    event_date: "",
    event_time: "",
    location_name: "",
    location_address: "",
    description: "",
    display_order: 0,
  };
  let formError = "";
  let formSubmitting = false;

  // Delete confirmation
  let deletingEvent = null;
  let deleteSubmitting = false;

  const eventTypes = [
    { value: "ceremony", label: "Ceremony" },
    { value: "reception", label: "Reception" },
    { value: "rehearsal", label: "Rehearsal Dinner" },
    { value: "welcome", label: "Welcome Party" },
    { value: "brunch", label: "Brunch" },
    { value: "other", label: "Other" },
  ];

  onMount(async () => {
    await loadEvents();
  });

  async function loadEvents() {
    loading = true;
    error = "";
    try {
      const result = await useListAdminEvents({ withCredentials: true });
      events = result.events;
    } catch (err) {
      error = getErrorMessage(err);
    } finally {
      loading = false;
    }
  }

  function resetForm() {
    formData = {
      name: "",
      event_type: "ceremony",
      event_date: "",
      event_time: "",
      location_name: "",
      location_address: "",
      description: "",
      display_order: events.length,
    };
  }

  function openAddForm() {
    editingEvent = null;
    resetForm();
    formError = "";
    showForm = true;
  }

  function openEditForm(event) {
    editingEvent = event;
    formData = {
      name: event.name,
      event_type: event.event_type,
      event_date: event.event_date,
      event_time: event.event_time,
      location_name: event.location_name,
      location_address: event.location_address,
      description: event.description || "",
      display_order: event.display_order,
    };
    formError = "";
    showForm = true;
  }

  function closeForm() {
    showForm = false;
    editingEvent = null;
    formError = "";
  }

  async function handleFormSubmit() {
    formError = "";
    if (!formData.name.trim()) {
      formError = "Event name is required";
      return;
    }
    if (!formData.event_date) {
      formError = "Event date is required";
      return;
    }
    if (!formData.event_time) {
      formError = "Event time is required";
      return;
    }
    if (!formData.location_name.trim()) {
      formError = "Location name is required";
      return;
    }
    if (!formData.location_address.trim()) {
      formError = "Location address is required";
      return;
    }

    formSubmitting = true;
    try {
      const payload = {
        name: formData.name.trim(),
        event_type: formData.event_type,
        event_date: formData.event_date,
        event_time: formData.event_time,
        location_name: formData.location_name.trim(),
        location_address: formData.location_address.trim(),
        description: formData.description.trim() || null,
        display_order: formData.display_order,
      };

      if (editingEvent) {
        await useUpdateEvent(editingEvent.id, payload, {
          withCredentials: true,
        });
      } else {
        await useCreateEvent(payload, { withCredentials: true });
      }
      closeForm();
      await loadEvents();
    } catch (err) {
      formError = getErrorMessage(err);
    } finally {
      formSubmitting = false;
    }
  }

  function openDeleteConfirm(event) {
    deletingEvent = event;
  }

  function closeDeleteConfirm() {
    deletingEvent = null;
  }

  async function handleDelete() {
    if (!deletingEvent) return;
    deleteSubmitting = true;
    try {
      await useDeleteEvent(deletingEvent.id, { withCredentials: true });
      closeDeleteConfirm();
      await loadEvents();
    } catch (err) {
      error = getErrorMessage(err);
      closeDeleteConfirm();
    } finally {
      deleteSubmitting = false;
    }
  }

  function formatDate(dateStr) {
    const date = new Date(dateStr + "T00:00:00");
    return date.toLocaleDateString("en-US", {
      weekday: "short",
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  }

  function formatTime(timeStr) {
    const [hours, minutes] = timeStr.split(":");
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- temporary Date for formatting only
    const date = new Date();
    date.setHours(parseInt(hours), parseInt(minutes));
    return date.toLocaleTimeString("en-US", {
      hour: "numeric",
      minute: "2-digit",
    });
  }
</script>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <p class="text-gray-600">{events.length} events</p>
    <button
      on:click={openAddForm}
      class="rounded-lg bg-blue-600 px-4 py-2 font-medium text-white hover:bg-blue-700"
    >
      Add Event
    </button>
  </div>

  <!-- Error message -->
  {#if error}
    <div class="rounded-lg bg-red-50 p-4 text-red-600">{error}</div>
  {/if}

  <!-- Loading state -->
  {#if loading}
    <div class="py-8 text-center">
      <p class="text-gray-500">Loading events...</p>
    </div>
  {:else if events.length === 0}
    <div
      class="rounded-lg border-2 border-dashed border-gray-300 py-12 text-center"
    >
      <p class="text-gray-500">No events yet. Add your first event.</p>
    </div>
  {:else}
    <!-- Event cards -->
    <div class="space-y-4">
      {#each events as event (event.id)}
        <div class="rounded-lg border border-gray-200 bg-white p-6 shadow-sm">
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-3">
                <h3 class="text-lg font-semibold text-gray-900">
                  {event.name}
                </h3>
                <span
                  class="inline-flex rounded-full bg-blue-100 px-2 py-0.5 text-xs font-medium text-blue-700"
                >
                  {event.event_type}
                </span>
              </div>
              <p class="mt-1 text-gray-600">
                {formatDate(event.event_date)} at {formatTime(event.event_time)}
              </p>
              <p class="mt-2 font-medium text-gray-700">
                {event.location_name}
              </p>
              <p class="text-sm text-gray-500">{event.location_address}</p>
              {#if event.description}
                <p class="mt-2 text-sm text-gray-600">{event.description}</p>
              {/if}
            </div>
            <div class="ml-4 flex gap-2">
              <button
                on:click={() => openEditForm(event)}
                class="rounded px-3 py-1 text-sm text-blue-600 hover:bg-blue-50"
              >
                Edit
              </button>
              <button
                on:click={() => openDeleteConfirm(event)}
                class="rounded px-3 py-1 text-sm text-red-600 hover:bg-red-50"
              >
                Delete
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Add/Edit Modal -->
{#if showForm}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center overflow-y-auto bg-black bg-opacity-50 p-4"
  >
    <div class="w-full max-w-lg rounded-lg bg-white p-6 shadow-xl">
      <h2 class="mb-4 text-lg font-semibold text-gray-900">
        {editingEvent ? "Edit Event" : "Add Event"}
      </h2>

      <form on:submit|preventDefault={handleFormSubmit} class="space-y-4">
        <div class="grid grid-cols-2 gap-4">
          <div class="col-span-2">
            <label
              for="name"
              class="mb-1 block text-sm font-medium text-gray-700"
            >
              Event Name
            </label>
            <input
              id="name"
              type="text"
              bind:value={formData.name}
              class="w-full rounded-lg border border-gray-300 px-3 py-2"
              placeholder="e.g., Wedding Ceremony"
            />
          </div>

          <div>
            <label
              for="eventType"
              class="mb-1 block text-sm font-medium text-gray-700"
            >
              Event Type
            </label>
            <select
              id="eventType"
              bind:value={formData.event_type}
              class="w-full rounded-lg border border-gray-300 px-3 py-2"
            >
              {#each eventTypes as type (type.value)}
                <option value={type.value}>{type.label}</option>
              {/each}
            </select>
          </div>

          <div>
            <label
              for="displayOrder"
              class="mb-1 block text-sm font-medium text-gray-700"
            >
              Display Order
            </label>
            <input
              id="displayOrder"
              type="number"
              bind:value={formData.display_order}
              min="0"
              class="w-full rounded-lg border border-gray-300 px-3 py-2"
            />
          </div>

          <div>
            <label
              for="eventDate"
              class="mb-1 block text-sm font-medium text-gray-700"
            >
              Date
            </label>
            <input
              id="eventDate"
              type="date"
              bind:value={formData.event_date}
              class="w-full rounded-lg border border-gray-300 px-3 py-2"
            />
          </div>

          <div>
            <label
              for="eventTime"
              class="mb-1 block text-sm font-medium text-gray-700"
            >
              Time
            </label>
            <input
              id="eventTime"
              type="time"
              bind:value={formData.event_time}
              class="w-full rounded-lg border border-gray-300 px-3 py-2"
            />
          </div>

          <div class="col-span-2">
            <label
              for="locationName"
              class="mb-1 block text-sm font-medium text-gray-700"
            >
              Location Name
            </label>
            <input
              id="locationName"
              type="text"
              bind:value={formData.location_name}
              class="w-full rounded-lg border border-gray-300 px-3 py-2"
              placeholder="e.g., Grand Ballroom"
            />
          </div>

          <div class="col-span-2">
            <label
              for="locationAddress"
              class="mb-1 block text-sm font-medium text-gray-700"
            >
              Location Address
            </label>
            <input
              id="locationAddress"
              type="text"
              bind:value={formData.location_address}
              class="w-full rounded-lg border border-gray-300 px-3 py-2"
              placeholder="e.g., 123 Main St, City, State 12345"
            />
          </div>

          <div class="col-span-2">
            <label
              for="description"
              class="mb-1 block text-sm font-medium text-gray-700"
            >
              Description (optional)
            </label>
            <textarea
              id="description"
              bind:value={formData.description}
              rows="3"
              class="w-full rounded-lg border border-gray-300 px-3 py-2"
              placeholder="Additional details about the event..."
            ></textarea>
          </div>
        </div>

        {#if formError}
          <div class="rounded-lg bg-red-50 p-3 text-sm text-red-600">
            {formError}
          </div>
        {/if}

        <div class="flex justify-end gap-3 pt-2">
          <button
            type="button"
            on:click={closeForm}
            class="rounded-lg border border-gray-300 px-4 py-2 text-gray-700 hover:bg-gray-50"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={formSubmitting}
            class="rounded-lg bg-blue-600 px-4 py-2 font-medium text-white hover:bg-blue-700 disabled:opacity-50"
          >
            {formSubmitting
              ? "Saving..."
              : editingEvent
                ? "Save Changes"
                : "Add Event"}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Delete Confirmation Modal -->
{#if deletingEvent}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50"
  >
    <div class="w-full max-w-md rounded-lg bg-white p-6 shadow-xl">
      <h2 class="mb-2 text-lg font-semibold text-gray-900">Delete Event</h2>
      <p class="mb-4 text-gray-600">
        Are you sure you want to delete <strong>{deletingEvent.name}</strong>?
        This action cannot be undone.
      </p>

      <div class="flex justify-end gap-3">
        <button
          on:click={closeDeleteConfirm}
          class="rounded-lg border border-gray-300 px-4 py-2 text-gray-700 hover:bg-gray-50"
        >
          Cancel
        </button>
        <button
          on:click={handleDelete}
          disabled={deleteSubmitting}
          class="rounded-lg bg-red-600 px-4 py-2 font-medium text-white hover:bg-red-700 disabled:opacity-50"
        >
          {deleteSubmitting ? "Deleting..." : "Delete"}
        </button>
      </div>
    </div>
  </div>
{/if}
