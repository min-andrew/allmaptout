<script lang="ts">
  import { onMount } from "svelte";
  import { useListGuests } from "../kubb/hooks/useListGuests";
  import { useCreateGuest } from "../kubb/hooks/useCreateGuest";
  import { useUpdateGuest } from "../kubb/hooks/useUpdateGuest";
  import { useDeleteGuest } from "../kubb/hooks/useDeleteGuest";
  import { useRegenerateCode } from "../kubb/hooks/useRegenerateCode";
  import { getErrorMessage } from "../api/errors";
  import type { AdminGuestResponse } from "../kubb/types/AdminGuestResponse";

  let guests: AdminGuestResponse[] = [];
  let loading = true;
  let error = "";

  // Add/Edit form state
  let showForm = false;
  let editingGuest: AdminGuestResponse | null = null;
  let formName = "";
  let formPartySize = 1;
  let formError = "";
  let formSubmitting = false;

  // Delete confirmation
  let deletingGuest: AdminGuestResponse | null = null;
  let deleteSubmitting = false;

  // Code regeneration
  let regeneratingId: string | null = null;

  onMount(async () => {
    await loadGuests();
  });

  async function loadGuests() {
    loading = true;
    error = "";
    try {
      const result = await useListGuests({ withCredentials: true });
      guests = result.guests;
    } catch (err) {
      error = getErrorMessage(err);
    } finally {
      loading = false;
    }
  }

  function openAddForm() {
    editingGuest = null;
    formName = "";
    formPartySize = 1;
    formError = "";
    showForm = true;
  }

  function openEditForm(guest: AdminGuestResponse) {
    editingGuest = guest;
    formName = guest.name;
    formPartySize = guest.party_size;
    formError = "";
    showForm = true;
  }

  function closeForm() {
    showForm = false;
    editingGuest = null;
    formError = "";
  }

  async function handleFormSubmit() {
    formError = "";
    if (!formName.trim()) {
      formError = "Name is required";
      return;
    }
    if (formPartySize < 1 || formPartySize > 20) {
      formError = "Party size must be between 1 and 20";
      return;
    }

    formSubmitting = true;
    try {
      if (editingGuest) {
        await useUpdateGuest(
          editingGuest.id,
          { name: formName.trim(), party_size: formPartySize },
          { withCredentials: true },
        );
      } else {
        await useCreateGuest(
          { name: formName.trim(), party_size: formPartySize },
          { withCredentials: true },
        );
      }
      closeForm();
      await loadGuests();
    } catch (err) {
      formError = getErrorMessage(err);
    } finally {
      formSubmitting = false;
    }
  }

  function openDeleteConfirm(guest: AdminGuestResponse) {
    deletingGuest = guest;
  }

  function closeDeleteConfirm() {
    deletingGuest = null;
  }

  async function handleDelete() {
    if (!deletingGuest) return;
    deleteSubmitting = true;
    try {
      await useDeleteGuest(deletingGuest.id, { withCredentials: true });
      closeDeleteConfirm();
      await loadGuests();
    } catch (err) {
      error = getErrorMessage(err);
      closeDeleteConfirm();
    } finally {
      deleteSubmitting = false;
    }
  }

  async function handleRegenerateCode(guest: AdminGuestResponse) {
    regeneratingId = guest.id;
    try {
      const result = await useRegenerateCode(guest.id, {
        withCredentials: true,
      });
      // Update the guest in the list with new code
      guests = guests.map((g) =>
        g.id === guest.id ? { ...g, invite_code: result.invite_code } : g,
      );
    } catch (err) {
      error = getErrorMessage(err);
    } finally {
      regeneratingId = null;
    }
  }

  function getRsvpStatusBadge(guest: AdminGuestResponse) {
    if (!guest.rsvp.has_responded) {
      return { class: "bg-gray-100 text-gray-700", text: "No Response" };
    }
    if (guest.rsvp.attending_count > 0) {
      return {
        class: "bg-green-100 text-green-700",
        text: `${guest.rsvp.attending_count} Attending`,
      };
    }
    return { class: "bg-red-100 text-red-700", text: "Declined" };
  }
</script>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <p class="text-gray-600">{guests.length} guests total</p>
    </div>
    <button
      on:click={openAddForm}
      class="rounded-lg bg-blue-600 px-4 py-2 font-medium text-white hover:bg-blue-700"
    >
      Add Guest
    </button>
  </div>

  <!-- Error message -->
  {#if error}
    <div class="rounded-lg bg-red-50 p-4 text-red-600">{error}</div>
  {/if}

  <!-- Loading state -->
  {#if loading}
    <div class="py-8 text-center">
      <p class="text-gray-500">Loading guests...</p>
    </div>
  {:else if guests.length === 0}
    <div
      class="rounded-lg border-2 border-dashed border-gray-300 py-12 text-center"
    >
      <p class="text-gray-500">
        No guests yet. Add your first guest to get started.
      </p>
    </div>
  {:else}
    <!-- Guest list -->
    <div
      class="overflow-hidden rounded-lg border border-gray-200 bg-white shadow"
    >
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th
              class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500"
            >
              Guest
            </th>
            <th
              class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500"
            >
              Party Size
            </th>
            <th
              class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500"
            >
              Invite Code
            </th>
            <th
              class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500"
            >
              RSVP Status
            </th>
            <th
              class="px-6 py-3 text-right text-xs font-medium uppercase tracking-wider text-gray-500"
            >
              Actions
            </th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          {#each guests as guest (guest.id)}
            {@const badge = getRsvpStatusBadge(guest)}
            <tr class="hover:bg-gray-50">
              <td class="whitespace-nowrap px-6 py-4">
                <div class="font-medium text-gray-900">{guest.name}</div>
              </td>
              <td class="whitespace-nowrap px-6 py-4 text-gray-500">
                {guest.party_size}
              </td>
              <td class="whitespace-nowrap px-6 py-4">
                {#if guest.invite_code}
                  <div class="flex items-center gap-2">
                    <code
                      class="rounded bg-gray-100 px-2 py-1 font-mono text-sm"
                    >
                      {guest.invite_code}
                    </code>
                    <button
                      on:click={() => handleRegenerateCode(guest)}
                      disabled={regeneratingId === guest.id}
                      class="text-sm text-blue-600 hover:text-blue-700 disabled:opacity-50"
                      title="Generate new code"
                    >
                      {regeneratingId === guest.id ? "..." : "Regenerate"}
                    </button>
                  </div>
                {:else}
                  <span class="text-gray-400">No code</span>
                {/if}
              </td>
              <td class="whitespace-nowrap px-6 py-4">
                <span
                  class="inline-flex rounded-full px-2 py-1 text-xs font-medium {badge.class}"
                >
                  {badge.text}
                </span>
              </td>
              <td class="whitespace-nowrap px-6 py-4 text-right">
                <button
                  on:click={() => openEditForm(guest)}
                  class="mr-3 text-sm text-blue-600 hover:text-blue-700"
                >
                  Edit
                </button>
                <button
                  on:click={() => openDeleteConfirm(guest)}
                  class="text-sm text-red-600 hover:text-red-700"
                >
                  Delete
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<!-- Add/Edit Modal -->
{#if showForm}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50"
  >
    <div class="w-full max-w-md rounded-lg bg-white p-6 shadow-xl">
      <h2 class="mb-4 text-lg font-semibold text-gray-900">
        {editingGuest ? "Edit Guest" : "Add Guest"}
      </h2>

      <form on:submit|preventDefault={handleFormSubmit} class="space-y-4">
        <div>
          <label
            for="name"
            class="mb-1 block text-sm font-medium text-gray-700"
          >
            Name
          </label>
          <input
            id="name"
            type="text"
            bind:value={formName}
            class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            placeholder="Guest name"
          />
        </div>

        <div>
          <label
            for="partySize"
            class="mb-1 block text-sm font-medium text-gray-700"
          >
            Party Size
          </label>
          <input
            id="partySize"
            type="number"
            bind:value={formPartySize}
            min="1"
            max="20"
            class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
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
              : editingGuest
                ? "Save Changes"
                : "Add Guest"}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Delete Confirmation Modal -->
{#if deletingGuest}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50"
  >
    <div class="w-full max-w-md rounded-lg bg-white p-6 shadow-xl">
      <h2 class="mb-2 text-lg font-semibold text-gray-900">Delete Guest</h2>
      <p class="mb-4 text-gray-600">
        Are you sure you want to delete <strong>{deletingGuest.name}</strong>?
        This will also delete their invite code and any RSVP data. This action
        cannot be undone.
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
