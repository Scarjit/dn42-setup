<script lang="ts">
  import { LogOut } from 'lucide-svelte';

  interface Props {
    onLogout: () => void;
  }

  let { onLogout }: Props = $props();
</script>

<button
  onclick={onLogout}
  class="px-4 py-2 bg-gray-600 text-white font-medium rounded-md hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 transition-colors flex items-center gap-2"
>
  <LogOut class="w-4 h-4" />
  <span>Logout</span>
</button>
