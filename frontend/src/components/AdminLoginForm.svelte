<script lang="ts">
  import { adminLogin } from "../api/auth";
  import { ApiError, NetworkError } from "../api/client";

  let username = "";
  let password = "";
  let error = "";
  let loading = false;

  async function handleSubmit() {
    error = "";

    if (!username.trim() || !password) {
      error = "Please enter username and password";
      return;
    }

    loading = true;
    try {
      await adminLogin(username.trim(), password);
      window.location.href = "/admin";
    } catch (err) {
      if (err instanceof ApiError) {
        if (err.isUnauthorized) {
          error = "Invalid username or password";
        } else {
          error = err.userMessage;
        }
      } else if (err instanceof NetworkError) {
        error = err.message;
      } else {
        error = "Something went wrong";
      }
    } finally {
      loading = false;
    }
  }
</script>

<form on:submit|preventDefault={handleSubmit} class="w-full max-w-sm">
  <div class="mb-4">
    <label for="username" class="mb-2 block text-sm font-medium text-gray-700">
      Username
    </label>
    <input
      id="username"
      type="text"
      bind:value={username}
      autocomplete="username"
      disabled={loading}
      class="w-full rounded-lg border border-gray-300 px-4 py-3 focus:border-transparent focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
    />
  </div>

  <div class="mb-4">
    <label for="password" class="mb-2 block text-sm font-medium text-gray-700">
      Password
    </label>
    <input
      id="password"
      type="password"
      bind:value={password}
      autocomplete="current-password"
      disabled={loading}
      class="w-full rounded-lg border border-gray-300 px-4 py-3 focus:border-transparent focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
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
    {loading ? "Signing in..." : "Sign in"}
  </button>
</form>
