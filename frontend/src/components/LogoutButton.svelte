<script lang="ts">
  import { useLogout } from "../kubb/hooks/useLogout";

  let loading = false;

  async function handleLogout() {
    loading = true;
    try {
      await useLogout({ withCredentials: true });
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
