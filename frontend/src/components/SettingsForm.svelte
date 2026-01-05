<script lang="ts">
  import { useChangePassword } from "../kubb/hooks/useChangePassword";
  import { getErrorMessage } from "../api/errors";

  let currentPassword = "";
  let newPassword = "";
  let confirmPassword = "";
  let error = "";
  let success = "";
  let submitting = false;

  async function handleSubmit() {
    error = "";
    success = "";

    if (!currentPassword) {
      error = "Current password is required";
      return;
    }
    if (!newPassword) {
      error = "New password is required";
      return;
    }
    if (newPassword.length < 8) {
      error = "New password must be at least 8 characters";
      return;
    }
    if (newPassword !== confirmPassword) {
      error = "Passwords do not match";
      return;
    }

    submitting = true;
    try {
      await useChangePassword(
        {
          current_password: currentPassword,
          new_password: newPassword,
        },
        { withCredentials: true },
      );
      success = "Password changed successfully";
      currentPassword = "";
      newPassword = "";
      confirmPassword = "";
    } catch (err) {
      error = getErrorMessage(err);
    } finally {
      submitting = false;
    }
  }
</script>

<div class="space-y-8">
  <!-- Password Change Section -->
  <div class="rounded-lg bg-white p-6 shadow">
    <h2 class="mb-4 text-lg font-semibold text-gray-900">Change Password</h2>

    <form on:submit|preventDefault={handleSubmit} class="max-w-md space-y-4">
      <div>
        <label
          for="currentPassword"
          class="mb-1 block text-sm font-medium text-gray-700"
        >
          Current Password
        </label>
        <input
          id="currentPassword"
          type="password"
          bind:value={currentPassword}
          class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
        />
      </div>

      <div>
        <label
          for="newPassword"
          class="mb-1 block text-sm font-medium text-gray-700"
        >
          New Password
        </label>
        <input
          id="newPassword"
          type="password"
          bind:value={newPassword}
          class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
        />
        <p class="mt-1 text-sm text-gray-500">Minimum 8 characters</p>
      </div>

      <div>
        <label
          for="confirmPassword"
          class="mb-1 block text-sm font-medium text-gray-700"
        >
          Confirm New Password
        </label>
        <input
          id="confirmPassword"
          type="password"
          bind:value={confirmPassword}
          class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
        />
      </div>

      {#if error}
        <div class="rounded-lg bg-red-50 p-3 text-sm text-red-600">{error}</div>
      {/if}

      {#if success}
        <div class="rounded-lg bg-green-50 p-3 text-sm text-green-600">
          {success}
        </div>
      {/if}

      <button
        type="submit"
        disabled={submitting}
        class="rounded-lg bg-blue-600 px-4 py-2 font-medium text-white hover:bg-blue-700 disabled:opacity-50"
      >
        {submitting ? "Changing..." : "Change Password"}
      </button>
    </form>
  </div>
</div>
