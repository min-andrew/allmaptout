<script lang="ts">
  import { useValidateCode } from "../kubb/hooks/useValidateCode";
  import { getErrorMessage } from "../api/errors";

  let code = "";
  let error = "";
  let loading = false;

  async function handleSubmit() {
    error = "";

    if (!code.trim()) {
      error = "Please enter a code";
      return;
    }

    loading = true;
    try {
      const result = await useValidateCode(
        { code: code.trim() },
        { withCredentials: true },
      );

      if (result.session_type === "guest") {
        window.location.href = "/events";
      } else if (result.session_type === "admin_pending") {
        window.location.href = "/admin/login";
      }
    } catch (err) {
      error = getErrorMessage(err);
    } finally {
      loading = false;
    }
  }
</script>

<form on:submit|preventDefault={handleSubmit} class="w-full max-w-sm">
  <div class="mb-4">
    <label for="code" class="mb-2 block text-sm font-medium text-gray-700">
      Enter your invite code
    </label>
    <input
      id="code"
      type="text"
      bind:value={code}
      placeholder="e.g., SMITH2024"
      disabled={loading}
      class="w-full rounded-lg border border-gray-300 px-4 py-3 text-lg focus:border-transparent focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
    />
  </div>

  {#if error}
    <p class="mb-4 text-sm text-red-600">{error}</p>
  {/if}

  <button
    type="submit"
    disabled={loading}
    class="w-full rounded-lg bg-blue-600 px-4 py-3 font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
  >
    {loading ? "Checking..." : "Continue"}
  </button>
</form>
