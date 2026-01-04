<script lang="ts">
  import { logout } from "../api/auth";

  let loading = false;

  async function handleLogout() {
    loading = true;
    try {
      await logout();
    } catch {
      // Ignore errors - redirect anyway
    }
    window.location.href = "/";
  }
</script>

<button
  on:click={handleLogout}
  disabled={loading}
  class="px-4 py-2 text-sm text-gray-600 hover:text-gray-900 disabled:opacity-50"
>
  {loading ? "..." : "Logout"}
</button>
