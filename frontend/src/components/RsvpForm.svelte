<script lang="ts">
  import { onMount } from "svelte";
  import { useGetRsvpStatus } from "../kubb/hooks/useGetRsvpStatus";
  import { useSubmitRsvp } from "../kubb/hooks/useSubmitRsvp";

  export let guestName = "";

  let status = null;
  let attendees = [];
  let loading = true;
  let submitting = false;
  let error = "";
  let success = "";

  const mealOptions = [
    { value: "beef", label: "Beef" },
    { value: "chicken", label: "Chicken" },
    { value: "fish", label: "Fish" },
    { value: "vegetarian", label: "Vegetarian" },
    { value: "vegan", label: "Vegan" },
  ];

  onMount(async () => {
    try {
      status = await useGetRsvpStatus({ withCredentials: true });
      if (status.rsvp) {
        attendees = status.rsvp.attendees.map((a) => ({
          name: a.name,
          is_attending: a.is_attending,
          meal_preference: a.meal_preference,
          dietary_restrictions: a.dietary_restrictions,
          is_primary: a.is_primary,
        }));
      } else {
        attendees = [
          {
            name: status.guest_name,
            is_attending: true,
            meal_preference: null,
            dietary_restrictions: null,
            is_primary: true,
          },
        ];
      }
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to load RSVP status";
    } finally {
      loading = false;
    }
  });

  function addAttendee() {
    if (status && attendees.length < status.party_size) {
      attendees = [
        ...attendees,
        {
          name: "",
          is_attending: true,
          meal_preference: null,
          dietary_restrictions: null,
          is_primary: false,
        },
      ];
    }
  }

  function removeAttendee(index) {
    if (!attendees[index].is_primary && attendees.length > 1) {
      attendees = attendees.filter((_, i) => i !== index);
    }
  }

  async function handleSubmit() {
    error = "";
    success = "";

    for (const att of attendees) {
      if (!att.name.trim()) {
        error = "All attendees must have a name";
        return;
      }
      if (att.is_attending && !att.meal_preference) {
        error = "Please select a meal preference for all attending guests";
        return;
      }
    }

    submitting = true;
    try {
      await useSubmitRsvp({ attendees }, { withCredentials: true });
      success = "Your RSVP has been submitted successfully!";
    } catch (err) {
      error = err instanceof Error ? err.message : "Failed to submit RSVP";
    } finally {
      submitting = false;
    }
  }
</script>

{#if loading}
  <div class="py-8 text-center">
    <p class="text-gray-500">Loading...</p>
  </div>
{:else if error && !status}
  <div class="rounded-lg bg-red-50 p-4 text-red-600">{error}</div>
{:else if status}
  <form on:submit|preventDefault={handleSubmit} class="space-y-6">
    {#if status.has_responded}
      <div class="rounded-lg bg-blue-50 p-4 text-blue-700">
        You have already responded. You can update your RSVP below.
      </div>
    {/if}

    <p class="text-gray-600">
      Party size: {status.party_size}
      {status.party_size === 1 ? "guest" : "guests"}
    </p>

    {#each attendees as attendee, index (index)}
      <div class="space-y-4 rounded-lg border border-gray-200 p-4">
        <div class="flex items-center justify-between">
          <h3 class="font-medium text-gray-900">
            {attendee.is_primary ? "Primary Guest" : `Guest ${index + 1}`}
          </h3>
          {#if !attendee.is_primary && attendees.length > 1}
            <button
              type="button"
              on:click={() => removeAttendee(index)}
              class="text-sm text-red-600 hover:text-red-700"
            >
              Remove
            </button>
          {/if}
        </div>

        <div>
          <label class="mb-1 block text-sm font-medium text-gray-700">
            Name
          </label>
          <input
            type="text"
            bind:value={attendee.name}
            disabled={attendee.is_primary}
            class="w-full rounded-lg border border-gray-300 px-3 py-2 disabled:bg-gray-100"
          />
        </div>

        <div>
          <label class="mb-2 block text-sm font-medium text-gray-700">
            Will you be attending?
          </label>
          <div class="flex gap-4">
            <label class="flex items-center">
              <input
                type="radio"
                bind:group={attendee.is_attending}
                value={true}
                class="mr-2"
              />
              Yes
            </label>
            <label class="flex items-center">
              <input
                type="radio"
                bind:group={attendee.is_attending}
                value={false}
                class="mr-2"
              />
              No
            </label>
          </div>
        </div>

        {#if attendee.is_attending}
          <div>
            <label class="mb-1 block text-sm font-medium text-gray-700">
              Meal Preference
            </label>
            <select
              bind:value={attendee.meal_preference}
              class="w-full rounded-lg border border-gray-300 px-3 py-2"
            >
              <option value={null}>Select a meal...</option>
              {#each mealOptions as option (option.value)}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </div>

          <div>
            <label class="mb-1 block text-sm font-medium text-gray-700">
              Dietary Restrictions (optional)
            </label>
            <textarea
              bind:value={attendee.dietary_restrictions}
              rows="2"
              placeholder="e.g., gluten-free, nut allergy"
              class="w-full rounded-lg border border-gray-300 px-3 py-2"
            ></textarea>
          </div>
        {/if}
      </div>
    {/each}

    {#if attendees.length < status.party_size}
      <button
        type="button"
        on:click={addAttendee}
        class="w-full rounded-lg border-2 border-dashed border-gray-300 py-3 text-gray-500 hover:border-gray-400 hover:text-gray-600"
      >
        + Add Guest ({attendees.length}/{status.party_size})
      </button>
    {/if}

    {#if error}
      <div class="rounded-lg bg-red-50 p-4 text-red-600">{error}</div>
    {/if}

    {#if success}
      <div class="rounded-lg bg-green-50 p-4 text-green-700">{success}</div>
    {/if}

    <button
      type="submit"
      disabled={submitting}
      class="w-full rounded-lg bg-blue-600 px-4 py-3 font-medium text-white hover:bg-blue-700 disabled:opacity-50"
    >
      {submitting ? "Submitting..." : "Submit RSVP"}
    </button>
  </form>
{/if}
